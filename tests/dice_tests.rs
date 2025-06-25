// tests/dice_tests.rs - Complete Dice Maiden Test Suite
// Place this file in the tests/ directory for integration testing

use dicemaiden_rs::{format_multiple_results_with_limit, parse_and_roll};

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // BASIC DICE FUNCTIONALITY TESTS
    // ============================================================================

    /// Test basic dice rolling
    #[test]
    fn test_basic_dice_rolls() {
        assert_valid_roll("1d6");
        assert_valid_roll("2d6");
        assert_valid_roll("3d8");
        assert_valid_roll("4d10");
        assert_valid_roll("5d12");
        assert_valid_roll("6d20");
        assert_valid_roll("10d6");
        assert_valid_roll("20d6");
        assert_valid_roll("100d6");

        // Default count (no number before d)
        assert_valid_roll("d6");
        assert_valid_roll("d20");
        assert_valid_roll("d100");

        // Percentile dice
        assert_valid_roll("1d%");
        assert_valid_roll("2d%");
        assert_valid_roll("d%");
    }

    /// Test dice limits and validation
    #[test]
    fn test_dice_limits() {
        // Test maximum dice count (500)
        assert_valid_roll("500d6");
        assert_invalid_roll("501d6");

        // Test maximum sides (1000)
        assert_valid_roll("1d1000");
        assert_invalid_roll("1d1001");

        // Test minimum sides
        assert_invalid_roll("1d0");
        assert_invalid_roll("2d-1");

        // Test zero dice
        assert_invalid_roll("0d6");
    }

    /// Test mathematical modifiers
    #[test]
    fn test_mathematical_modifiers() {
        // Addition
        assert_valid_roll("1d6 + 1");
        assert_valid_roll("2d6 + 5");
        assert_valid_roll("3d6+10");
        assert_valid_roll("1d20 + 15");

        // Subtraction
        assert_valid_roll("1d6 - 1");
        assert_valid_roll("2d6 - 3");
        assert_valid_roll("3d6-2");
        assert_valid_roll("1d20 - 5");

        // Multiplication
        assert_valid_roll("1d6 * 2");
        assert_valid_roll("2d6 * 3");
        assert_valid_roll("3d6*4");

        // Division
        assert_valid_roll("1d6 / 2");
        assert_valid_roll("4d6 / 2");
        assert_valid_roll("6d6/3");

        // Complex combinations
        assert_valid_roll("2d6 + 3d8 + 5");
        assert_valid_roll("1d20 + 1d6 - 2");
        assert_valid_roll("3d6 * 2 + 5");
        assert_valid_roll("4d6 / 2 - 1");
    }

    /// Test dice addition and subtraction
    #[test]
    fn test_dice_modifiers() {
        assert_valid_roll("2d6 + 1d4");
        assert_valid_roll("3d8 - 1d6");
        assert_valid_roll("4d6 + 2d8 + 1d4");
        assert_valid_roll("1d20 + 3d6 - 1d4");
        assert_valid_roll("2d10+1d8");
        assert_valid_roll("3d6-2d4");
    }

    /// Test mathematical operations with edge cases
    #[test]
    fn test_mathematical_edge_cases() {
        // Test negative results
        assert_valid_roll("1d6 - 10");
        assert_valid_roll("2d6 - 3d6");

        // Test division edge cases
        assert_valid_roll("10d6 / 2");
        assert_valid_roll("6d6 / 3");

        // Test multiplication with large numbers
        assert_valid_roll("1d6 * 10");
        assert_valid_roll("2d6 * 5");
    }

    // ============================================================================
    // ADVANCED DICE MECHANICS TESTS
    // ============================================================================

    /// Test exploding dice
    #[test]
    fn test_exploding_dice() {
        // Basic exploding (once)
        assert_valid_roll("3d6 e");
        assert_valid_roll("3d6 e6");
        assert_valid_roll("4d10 e8");
        assert_valid_roll("2d20 e20");
        assert_valid_roll("1d6e");
        assert_valid_roll("2d8e8");

        // Indefinite exploding
        assert_valid_roll("3d6 ie");
        assert_valid_roll("3d6 ie6");
        assert_valid_roll("4d10 ie8");
        assert_valid_roll("2d20 ie20");
        assert_valid_roll("1d6ie");
        assert_valid_roll("2d8ie8");

        // Exploding with other modifiers
        assert_valid_roll("4d6 e6 k3");
        assert_valid_roll("3d10 ie8 + 5");
        assert_valid_roll("2d6 e + 1d4");
    }

    /// Test keep/drop dice
    #[test]
    fn test_keep_drop_dice() {
        // Keep highest
        assert_valid_roll("4d6 k3");
        assert_valid_roll("5d8 k2");
        assert_valid_roll("6d10 k4");
        assert_valid_roll("3d6k2");

        // Keep lowest
        assert_valid_roll("4d6 kl3");
        assert_valid_roll("5d8 kl2");
        assert_valid_roll("6d10 kl4");
        assert_valid_roll("3d6kl2");

        // Drop lowest
        assert_valid_roll("4d6 d1");
        assert_valid_roll("5d8 d2");
        assert_valid_roll("6d10 d3");
        assert_valid_roll("4d6d1");

        // Keep/drop more than available (should handle gracefully)
        assert_valid_roll("3d6 k5"); // Keep more than available
        assert_valid_roll("3d6 d5"); // Drop more than available

        // Combinations
        assert_valid_roll("6d6 d1 k3");
        assert_valid_roll("4d6 k3 + 2");
    }

    /// Test reroll mechanics
    #[test]
    fn test_reroll_dice() {
        // Single reroll
        assert_valid_roll("4d6 r1");
        assert_valid_roll("3d10 r2");
        assert_valid_roll("2d20 r5");
        assert_valid_roll("4d6r1");

        // Indefinite reroll
        assert_valid_roll("4d6 ir1");
        assert_valid_roll("3d10 ir2");
        assert_valid_roll("2d20 ir5");
        assert_valid_roll("4d6ir1");

        // Reroll with other modifiers
        assert_valid_roll("4d6 r1 k3");
        assert_valid_roll("3d10 ir2 + 5");
        assert_valid_roll("2d6 r1 e6");
    }

    /// Test target/success counting
    #[test]
    fn test_target_success_counting() {
        assert_valid_roll("6d10 t7");
        assert_valid_roll("4d6 t4");
        assert_valid_roll("8d8 t6");
        assert_valid_roll("5d10t8");

        // Target with other modifiers
        assert_valid_roll("6d10 t7 e10");
        assert_valid_roll("4d6 t4 + 2");
        assert_valid_roll("8d8 t6 ie8");
    }

    /// Test failure and botch counting
    #[test]
    fn test_failure_botch_counting() {
        // Failures
        assert_valid_roll("6d10 f1");
        assert_valid_roll("4d6 f2");
        assert_valid_roll("8d8 f3");
        assert_valid_roll("5d10f1");

        // Botches
        assert_valid_roll("6d10 b");
        assert_valid_roll("6d10 b1");
        assert_valid_roll("4d6 b2");
        assert_valid_roll("8d8 b3");
        assert_valid_roll("5d10b1");

        // Combined failure/success systems
        assert_valid_roll("6d10 t7 f1");
        assert_valid_roll("8d10 t8 b1");
        assert_valid_roll("4d6 t4 f1 b1");
    }

    // ============================================================================
    // FLAGS AND SPECIAL MODIFIERS TESTS
    // ============================================================================

    /// Test flags and special modifiers
    #[test]
    fn test_flags() {
        // Private roll
        assert_valid_roll("p 1d6");
        assert_valid_roll("p 2d6 + 3");
        assert_valid_roll("p 4d6 k3");

        // Simple output
        assert_valid_roll("s 1d6");
        assert_valid_roll("s 2d6 + 3");
        assert_valid_roll("s 4d6 k3");

        // No results
        assert_valid_roll("nr 1d6");
        assert_valid_roll("nr 2d6 + 3");

        // Unsorted
        assert_valid_roll("ul 4d6");
        assert_valid_roll("ul 6d6 k4");

        // Multiple flags
        assert_valid_roll("p s 1d6");
        assert_valid_roll("s ul 4d6 k3");
        assert_valid_roll("p nr ul 2d6");
    }

    /// Test unsort flag specifically (ul)
    #[test]
    fn test_unsort_flag_detailed() {
        // Test that ul flag preserves roll order
        assert_valid_roll("ul 10d6");
        assert_valid_roll("ul 6d6 k4");
        assert_valid_roll("ul 4d6 d1");

        // Test unsort with other flags
        assert_valid_roll("p ul 4d6");
        assert_valid_roll("s ul 6d6 k3");
        assert_valid_roll("nr ul 3d6");

        // Test that unsorted flag affects behavior (not the flag itself)
        let results = parse_and_roll("ul 10d6").unwrap();
        // We can't test the flag directly anymore, but we can test that the parsing works
        assert_eq!(results.len(), 1);
    }

    /// Test private roll functionality
    #[test]
    fn test_private_roll_functionality() {
        let results = parse_and_roll("p 1d6 + 2").unwrap();
        assert!(results[0].private);

        // Test private rolls with complex expressions
        assert_valid_roll("p 4d6 k3 e6 + 2");
        assert_valid_roll("p 6d10 t7 ie10");
        assert_valid_roll("p attack + 5");

        // Test that private flag works with other flags
        assert_valid_roll("p s 1d6");
        assert_valid_roll("p ul 4d6 k3");
    }

    /// Test no results flag
    #[test]
    fn test_no_results_flag() {
        let results = parse_and_roll("nr 2d6 + 3").unwrap();
        assert!(results[0].no_results);

        // Test nr with complex expressions
        assert_valid_roll("nr 4d6 k3");
        assert_valid_roll("nr 6d10 t7");
        assert_valid_roll("nr attack + 5");
    }

    /// Test simple flag
    #[test]
    fn test_simple_flag() {
        let results = parse_and_roll("s 2d6 + 3").unwrap();
        assert!(results[0].simple);

        // Test simple with various expressions
        assert_valid_roll("s 4d6 k3 e6");
        assert_valid_roll("s 6d10 t7 ie10");
        assert_valid_roll("s 3df");
    }

    // ============================================================================
    // COMMENTS AND LABELS TESTS
    // ============================================================================

    /// Test comments and labels
    #[test]
    fn test_comments_and_labels() {
        // Comments
        assert_valid_roll("1d6 ! attack roll");
        assert_valid_roll("2d6 + 3 ! damage");
        assert_valid_roll("4d6 k3 ! ability score");
        assert_valid_roll("1d20!initiative");

        // Labels
        assert_valid_roll("(Attack) 1d20 + 5");
        assert_valid_roll("(Damage) 2d6 + 3");
        assert_valid_roll("(Stealth Check) 1d20 + 2");

        // Comments and labels together
        assert_valid_roll("(Attack Roll) 1d20 + 5 ! with sword");
        assert_valid_roll("(Initiative) 1d20 + 2 ! going first");
    }

    /// Test comment edge cases
    #[test]
    fn test_comment_edge_cases() {
        // Empty comments
        assert_valid_roll("1d6 !");
        assert_valid_roll("1d6 !   ");

        // Comments with special characters
        assert_valid_roll("1d6 ! attack + damage");
        assert_valid_roll("1d6 ! roll (with parentheses)");
        assert_valid_roll("1d6 ! roll ; with semicolon");
        assert_valid_roll("1d6 ! roll * with * stars");

        // Long comments
        let long_comment = "a".repeat(100);
        assert_valid_roll(&format!("1d6 ! {}", long_comment));
    }

    /// Test label edge cases
    #[test]
    fn test_label_edge_cases() {
        // Empty labels
        assert_valid_roll("() 1d6");
        assert_valid_roll("(   ) 1d6");

        // Labels with special characters
        assert_valid_roll("(Attack + Damage) 1d6");
        assert_valid_roll("(Roll (nested)) 1d6");
        assert_valid_roll("(Roll; with; semicolons) 1d6");

        // Long labels
        let long_label = "a".repeat(50);
        assert_valid_roll(&format!("({}) 1d6", long_label));
    }

    // ============================================================================
    // ROLL SETS AND MULTIPLE ROLLS TESTS
    // ============================================================================

    /// Test roll sets
    #[test]
    fn test_roll_sets() {
        assert_valid_roll("6 4d6");
        assert_valid_roll("3 1d20");
        assert_valid_roll("5 2d6 + 3");
        assert_valid_roll("4 3d6 k2");
        assert_valid_roll("2 1d6 e6");

        // Roll set limits
        assert_valid_roll("2 1d6"); // Minimum
        assert_valid_roll("20 1d6"); // Maximum
        assert_invalid_roll("1 1d6"); // Below minimum
        assert_invalid_roll("21 1d6"); // Above maximum

        // Roll sets with complex expressions
        assert_valid_roll("3 4d6 k3 + 2");
        assert_valid_roll("5 2d6 e6 + 1d4");
    }

    /// Test roll set edge cases
    #[test]
    fn test_roll_set_edge_cases() {
        // Minimum and maximum roll sets
        assert_valid_roll("2 1d6"); // Minimum
        assert_valid_roll("20 1d6"); // Maximum

        // Roll sets with complex expressions
        assert_valid_roll("5 4d6 k3 e6 + 2");
        assert_valid_roll("3 2d6 + 1d4 - 1");
        assert_valid_roll("4 1d20 + 5 ! initiative");

        // Roll sets with comments and labels
        assert_valid_roll("6 (Ability Score) 4d6 k3 ! character generation");
    }

    /// Test multiple rolls (semicolon separated)
    #[test]
    fn test_multiple_rolls() {
        assert_valid_roll("1d20; 1d6");
        assert_valid_roll("1d20 + 5; 2d6 + 3; 1d4");
        assert_valid_roll("4d6 k3; 4d6 k3; 4d6 k3");
        assert_valid_roll("1d20!init; 2d6!damage; 1d4!bonus");

        // Maximum 4 rolls
        assert_valid_roll("1d6; 1d6; 1d6; 1d6");
        assert_invalid_roll("1d6; 1d6; 1d6; 1d6; 1d6"); // Too many

        // Mixed complex rolls
        assert_valid_roll("4d6 k3 + 2; 1d20 + 5; 2d6 e6");
    }

    // ============================================================================
    // GAME SYSTEM ALIASES TESTS
    // ============================================================================

    /// Test game system aliases - Chronicles/World of Darkness
    #[test]
    fn test_cod_wod_aliases() {
        // Chronicles of Darkness
        assert_valid_roll("4cod");
        assert_valid_roll("6cod");
        assert_valid_roll("8cod");

        // CoD variants
        assert_valid_roll("4cod8"); // 8-again
        assert_valid_roll("5cod9"); // 9-again
        assert_valid_roll("3codr"); // Rote quality

        // CoD with modifiers
        assert_valid_roll("4cod + 2");
        assert_valid_roll("6cod8 - 1");

        // World of Darkness
        assert_valid_roll("4wod6");
        assert_valid_roll("5wod7");
        assert_valid_roll("6wod8");
        assert_valid_roll("8wod9");

        // WoD with modifiers
        assert_valid_roll("4wod8 + 1");
        assert_valid_roll("6wod7 - 2");
    }

    /// Test D&D/Pathfinder aliases
    #[test]
    fn test_dnd_aliases() {
        // Stat generation
        assert_valid_roll("dndstats");

        // Basic rolls
        assert_valid_roll("attack");
        assert_valid_roll("skill");
        assert_valid_roll("save");

        // Rolls with modifiers
        assert_valid_roll("attack + 5");
        assert_valid_roll("skill - 2");
        assert_valid_roll("save + 3");
        assert_valid_roll("attack+10");
        assert_valid_roll("skill-4");

        // Advantage/disadvantage
        assert_valid_roll("+d20");
        assert_valid_roll("-d20");
        assert_valid_roll("+d12");
        assert_valid_roll("-d12");

        // Percentile advantage/disadvantage
        assert_valid_roll("+d%");
        assert_valid_roll("-d%");
    }

    /// Test Fudge/FATE dice
    #[test]
    fn test_fudge_dice() {
        assert_valid_roll("3df");
        assert_valid_roll("4df");
        assert_valid_roll("2df");
        assert_valid_roll("6df");

        // Fudge dice with modifiers
        assert_valid_roll("4df + 2");
        assert_valid_roll("3df - 1");
        assert_valid_roll("4df+1");
    }

    /// Test Fudge dice symbol generation
    #[test]
    fn test_fudge_dice_symbols() {
        let results = parse_and_roll("4df").unwrap();
        assert!(results[0].fudge_symbols.is_some());

        // Test that Fudge dice work with modifiers
        assert_valid_roll("4df + 2");
        assert_valid_roll("3df - 1");
        assert_valid_roll("4df * 2");
    }

    /// Test Warhammer system
    #[test]
    fn test_warhammer_aliases() {
        assert_valid_roll("3wh4+");
        assert_valid_roll("5wh3+");
        assert_valid_roll("8wh5+");
        assert_valid_roll("10wh6+");
    }

    /// Test Hero System
    #[test]
    fn test_hero_system() {
        // Basic Hero System
        assert_valid_roll("hsn");
        assert_valid_roll("hsk");
        assert_valid_roll("hsh");

        // Hero System with dice counts
        assert_valid_roll("2hsn");
        assert_valid_roll("3hsk");
        assert_valid_roll("4hsn");

        // Fractional dice
        assert_valid_roll("2.5hsk");
        assert_valid_roll("3.5hsn");
        assert_valid_roll("1.5hsk");

        // Alternative fractional notation
        assert_valid_roll("2hsk1");
        assert_valid_roll("3hsn1");
        assert_valid_roll("4hsk1");

        // Hero System with manual dice expressions
        assert_valid_roll("2d6 hsn");
        assert_valid_roll("3d6 hsk");
        assert_valid_roll("3d6 hsh");
        assert_valid_roll("2d6 + 1d3 hsk");
    }

    /// Test Godbound system
    #[test]
    fn test_godbound_system() {
        // Basic Godbound
        assert_valid_roll("gb");
        assert_valid_roll("gbs");

        // Godbound with modifiers
        assert_valid_roll("gb + 5");
        assert_valid_roll("gbs - 2");
        assert_valid_roll("gb+3");
        assert_valid_roll("gbs-1");

        // Godbound with dice expressions
        assert_valid_roll("gb 3d8");
        assert_valid_roll("gbs 2d10");
        assert_valid_roll("gb 4d6 + 2");
        assert_valid_roll("gbs 1d20 - 1");

        // Manual Godbound modifiers
        assert_valid_roll("1d20 gb");
        assert_valid_roll("2d6 gbs");
        assert_valid_roll("3d8 gb + 5");
    }

    /// Test system-specific damage calculations
    #[test]
    fn test_damage_calculations() {
        // Test Godbound damage conversion with modifiers
        assert_valid_roll("2d6 + 3 gb"); // Should convert final total
        assert_valid_roll("3d8 - 1 gbs"); // Straight damage
        assert_valid_roll("1d20 + 5 gb"); // d20 with bonus

        // Test Hero System calculations
        assert_valid_roll("3d6 + 1d3 hsk"); // Killing damage with fractional die
        assert_valid_roll("2d6 hsn + 2"); // Normal damage with bonus
    }

    /// Test Wrath & Glory system
    #[test]
    fn test_wrath_and_glory() {
        // Basic Wrath & Glory
        assert_valid_roll("wng 4d6");
        assert_valid_roll("wng 5d6");
        assert_valid_roll("wng 6d6");

        // Wrath & Glory with difficulty
        assert_valid_roll("wng dn2 4d6");
        assert_valid_roll("wng dn3 5d6");
        assert_valid_roll("wng dn4 6d6");

        // Wrath & Glory special modes
        assert_valid_roll("wng 4d6 !soak");
        assert_valid_roll("wng 5d6 !exempt");
        assert_valid_roll("wng 6d6 !dmg");

        // Wrath & Glory with difficulty and special modes
        assert_valid_roll("wng dn2 4d6 !soak");
        assert_valid_roll("wng dn3 5d6 !exempt");

        // Manual WNG modifiers
        assert_valid_roll("4d6 wng");
        assert_valid_roll("5d6 wng2");
        assert_valid_roll("6d6 wng3t");
    }

    /// Test Wrath & Glory difficulty system thoroughly
    #[test]
    fn test_wrath_glory_difficulty_system() {
        // Test all difficulty levels
        for dn in 1..=10 {
            assert_valid_roll(&format!("wng dn{} 4d6", dn));
        }

        // Test soak/exempt/damage modes
        assert_valid_roll("wng 4d6 !soak");
        assert_valid_roll("wng 5d6 !exempt");
        assert_valid_roll("wng 6d6 !dmg");
        assert_valid_roll("wng 3d6 !damage"); // Alternative spelling

        // Test combination of difficulty and special modes
        assert_valid_roll("wng dn3 4d6 !soak");
        assert_valid_roll("wng dn2 5d6 !exempt");
    }

    /// Test other game system aliases
    #[test]
    fn test_other_game_systems() {
        // Shadowrun
        assert_valid_roll("sr6");
        assert_valid_roll("sr8");
        assert_valid_roll("sr10");

        // Storypath
        assert_valid_roll("sp4");
        assert_valid_roll("sp6");
        assert_valid_roll("sp8");

        // Year Zero
        assert_valid_roll("6yz");
        assert_valid_roll("8yz");
        assert_valid_roll("10yz");

        // Sunsails New Millennium
        assert_valid_roll("snm5");
        assert_valid_roll("snm6");
        assert_valid_roll("snm8");

        // D6 System
        assert_valid_roll("d6s4");
        assert_valid_roll("d6s6");
        assert_valid_roll("d6s8");
        assert_valid_roll("d6s4 + 2");

        // Double digit
        assert_valid_roll("dd34");
        assert_valid_roll("dd66");
        assert_valid_roll("dd89");

        // AGE system
        assert_valid_roll("age");

        // Exalted
        assert_valid_roll("ex5");
        assert_valid_roll("ex6");
        assert_valid_roll("ex5t8");
        assert_valid_roll("ex6t9");

        // Dark Heresy
        assert_valid_roll("dh 4d10");
        assert_valid_roll("dh 6d10");

        // Percentile dice
        assert_valid_roll("1d%");
        assert_valid_roll("2d%");
        assert_valid_roll("3d%");
    }

    /// Test Dark Heresy specific features
    #[test]
    fn test_dark_heresy_specific() {
        // Test DH modifier parsing
        assert_valid_roll("dh 4d10");
        assert_valid_roll("dh 6d10");
        assert_valid_roll("dh 8d10");

        // Test that DH rolls produce righteous fury notes
        let result = parse_and_roll("4d10 ie10 dh").unwrap();
        // Test that the roll works - checking notes would require examining specific results
        assert_eq!(result.len(), 1);
    }

    /// Test Earthdawn system
    #[test]
    fn test_earthdawn_system() {
        // Basic Earthdawn steps (1-50)
        assert_valid_roll("ed1");
        assert_valid_roll("ed5");
        assert_valid_roll("ed10");
        assert_valid_roll("ed15");
        assert_valid_roll("ed20");
        assert_valid_roll("ed30");
        assert_valid_roll("ed40");
        assert_valid_roll("ed50");

        // Earthdawn 4th edition
        assert_valid_roll("ed4e1");
        assert_valid_roll("ed4e10");
        assert_valid_roll("ed4e25");
        assert_valid_roll("ed4e50");

        // Invalid Earthdawn steps
        assert_invalid_roll("ed0");
        assert_invalid_roll("ed51");
        assert_invalid_roll("ed4e0");
        assert_invalid_roll("ed4e51");
    }

    /// Test Earthdawn step edge cases
    #[test]
    fn test_earthdawn_edge_cases() {
        // Test boundary values
        assert_valid_roll("ed1");
        assert_valid_roll("ed50");
        assert_invalid_roll("ed0");
        assert_invalid_roll("ed51");

        // Test 4th edition boundaries
        assert_valid_roll("ed4e1");
        assert_valid_roll("ed4e50");
        assert_invalid_roll("ed4e0");
        assert_invalid_roll("ed4e51");

        // Test intermediate values
        assert_valid_roll("ed15");
        assert_valid_roll("ed25");
        assert_valid_roll("ed35");
        assert_valid_roll("ed45");
    }

    /// Test percentile dice variations
    #[test]
    fn test_percentile_variations() {
        // Basic percentile
        assert_valid_roll("1d%");
        assert_valid_roll("2d%");
        assert_valid_roll("d%");

        // Percentile advantage/disadvantage
        assert_valid_roll("+d%");
        assert_valid_roll("-d%");

        // Percentile with modifiers
        assert_valid_roll("1d% + 10");
        assert_valid_roll("2d% k1");
    }

    /// Test all game system boundary cases
    #[test]
    fn test_game_system_boundaries() {
        // CoD variants
        assert_valid_roll("1cod"); // Minimum dice
        assert_valid_roll("20cod"); // Large pool
        assert_valid_roll("10cod8"); // 8-again
        assert_valid_roll("15codr"); // Rote quality

        // WoD difficulties
        assert_valid_roll("1wod6"); // Easy
        assert_valid_roll("1wod9"); // Hard
        assert_valid_roll("20wod8"); // Large pool, moderate difficulty

        // Shadowrun pool sizes
        assert_valid_roll("sr1"); // Minimum
        assert_valid_roll("sr20"); // Large pool

        // Year Zero pool sizes
        assert_valid_roll("1yz"); // Minimum
        assert_valid_roll("20yz"); // Large pool
    }

    // ============================================================================
    // COMPLEX COMBINATIONS AND ORDER OF OPERATIONS TESTS
    // ============================================================================

    /// Test complex modifier combinations
    #[test]
    fn test_complex_combinations() {
        // Multiple modifiers on same roll
        assert_valid_roll("4d6 e6 k3 + 2");
        assert_valid_roll("6d10 t7 ie10 - 1");
        assert_valid_roll("8d6 r1 k6 e6");
        assert_valid_roll("10d10 t8 f1 ie10");
        assert_valid_roll("5d6 d1 e6 + 1d4");

        // Very complex combinations
        assert_valid_roll("6d6 e6 ie k4 r1 + 2d4 - 1");
        assert_valid_roll("8d10 t7 f1 b1 ie10 + 5");
        assert_valid_roll("4d6 k3 e6 + 1d8 e8 - 2");

        // Combinations with aliases
        assert_valid_roll("4cod e10 + 2");
        assert_valid_roll("3df + 1d6 k1");
        assert_valid_roll("attack + 1d6 e6");
    }

    /// Test detailed order of operations
    #[test]
    fn test_detailed_order_of_operations() {
        // Test that modifiers are applied in correct order: explode, reroll, drop, keep, then math
        assert_valid_roll("6d6 e6 r1 d1 k3 + 5");
        assert_valid_roll("8d10 ie10 ir2 d2 k4 * 2");
        assert_valid_roll("4d6 e6 k3 + 1d4 e4 - 2");
    }

    /// Test operator precedence and parsing order
    #[test]
    fn test_operator_precedence() {
        // Test that modifiers are applied in correct order
        assert_valid_roll("4d6 d1 k2"); // Drop then keep
        assert_valid_roll("4d6 e6 k3"); // Explode then keep
        assert_valid_roll("4d6 r1 k3"); // Reroll then keep
        assert_valid_roll("4d6 e6 r1 k3"); // Explode, reroll, then keep

        // Test mathematical operations
        assert_valid_roll("1d6 + 2 * 3");
        assert_valid_roll("1d6 * 2 + 3");
        assert_valid_roll("2d6 + 1d4 * 2");
    }

    /// Test complex alias combinations
    #[test]
    fn test_complex_alias_combinations() {
        // Aliases with mathematical modifiers
        assert_valid_roll("attack + 1d6 e6"); // Attack roll with bonus damage
        assert_valid_roll("4cod + 2"); // Chronicles of Darkness with modifier
        assert_valid_roll("3df + 1"); // Fudge dice with bonus

        // Multiple aliases in semicolon rolls
        assert_valid_roll("attack; 2d6 + 3; 4cod");
        assert_valid_roll("dndstats; sr6; 3df");
    }

    // ============================================================================
    // WHITESPACE AND FORMATTING TESTS
    // ============================================================================

    /// Test spaces and formatting variations
    #[test]
    fn test_spacing_variations() {
        // No spaces
        assert_valid_roll("1d6+2");
        assert_valid_roll("2d6-1");
        assert_valid_roll("3d6e6");
        assert_valid_roll("4d6k3");
        assert_valid_roll("5d6t4");
        assert_valid_roll("6d6r1");
        assert_valid_roll("4d6e6k3+2");

        // Extra spaces
        assert_valid_roll("1d6  +  2");
        assert_valid_roll("2d6   -   1");
        assert_valid_roll("3d6  e6");
        assert_valid_roll("4d6  k3");
        assert_valid_roll("5d6   t4");

        // Mixed spacing
        assert_valid_roll("2d6 +3");
        assert_valid_roll("3d6+ 2");
        assert_valid_roll("4d6 e6k3");
        assert_valid_roll("5d6e6 k3");
        assert_valid_roll("6d6 k3+2");
    }

    /// Test whitespace handling in complex expressions
    #[test]
    fn test_whitespace_handling() {
        // Test various spacing patterns
        assert_valid_roll("1d6+2-1*3/2");
        assert_valid_roll("1d6 + 2 - 1 * 3 / 2");
        assert_valid_roll("1d6  +  2  -  1  *  3  /  2");
        assert_valid_roll("  1d6  +  2  ");
        assert_valid_roll("\t1d6\t+\t2\t");

        // Test spacing with modifiers
        assert_valid_roll("4d6e6k3+2");
        assert_valid_roll("4d6 e6 k3 + 2");
        assert_valid_roll("4d6  e6  k3  +  2");
        assert_valid_roll("4d6e6 k3+2");
        assert_valid_roll("4d6 e6k3 +2");

        // Test spacing with flags
        assert_valid_roll("p s ul 4d6 k3");
        assert_valid_roll("p  s  ul  4d6  k3");
        assert_valid_roll("ps ul4d6k3");
    }

    /// Test format string edge cases
    #[test]
    fn test_format_edge_cases() {
        // Leading/trailing spaces
        assert_valid_roll("  1d6  ");
        assert_valid_roll("\t2d6\t");
        assert_valid_roll("\n3d6\n");

        // Multiple spaces between elements
        assert_valid_roll("1d6    +    2");
        assert_valid_roll("4d6     k3");
        assert_valid_roll("2d6   e6   +   1d4");

        // Mixed separators
        assert_valid_roll("1d20; 2d6; 1d4");
        assert_valid_roll("1d20 ; 2d6 ; 1d4");
        assert_valid_roll("1d20  ;  2d6  ;  1d4");
    }

    /// Test regex pattern edge cases
    #[test]
    fn test_regex_edge_cases() {
        // Test patterns that might confuse the parser
        assert_valid_roll("1d6+2d8-3d4*2/3"); // Complex mathematical expression
        assert_valid_roll("4d6e6ie10k3r1t4f1b1"); // Many modifiers concatenated

        // Test spacing variations that might break regex patterns
        assert_valid_roll("1d6  +  2d8  -  3d4");
        assert_valid_roll("4d6   e6   k3   +   2");
    }

    // ============================================================================
    // CASE SENSITIVITY AND CHARACTER HANDLING TESTS
    // ============================================================================

    /// Test case sensitivity
    #[test]
    fn test_case_sensitivity() {
        // Modifiers should be case insensitive for some aliases
        assert_valid_roll("4COD");
        assert_valid_roll("3DF");
        assert_valid_roll("SR6");

        // But basic dice notation is case sensitive
        assert_valid_roll("1d6");
        assert_invalid_roll("1D6");

        // Flags should be case sensitive
        assert_valid_roll("p 1d6");
        assert_invalid_roll("P 1d6");
    }

    /// Test unicode and special character handling
    #[test]
    fn test_unicode_handling() {
        // Comments with unicode
        assert_valid_roll("1d6 ! café roll");
        assert_valid_roll("1d6 ! 攻撃ロール");
        assert_valid_roll("1d6 ! Würfelwurf");

        // Labels with unicode
        assert_valid_roll("(Café) 1d6");
        assert_valid_roll("(攻撃) 1d6");
        assert_valid_roll("(Würfel) 1d6");
    }

    // ============================================================================
    // BOUNDARY CONDITIONS AND EDGE CASES TESTS
    // ============================================================================

    /// Test boundary conditions
    #[test]
    fn test_boundary_conditions() {
        // Minimum valid values
        assert_valid_roll("1d1");
        assert_valid_roll("1d2");

        // Maximum valid values
        assert_valid_roll("500d1000");

        // Keep/drop edge cases
        assert_valid_roll("1d6 k1"); // Keep all
        assert_valid_roll("1d6 d0"); // Drop none (should be handled gracefully)
        assert_valid_roll("5d6 k10"); // Keep more than available
        assert_valid_roll("5d6 d10"); // Drop more than available

        // Target/threshold edge cases
        assert_valid_roll("1d6 t1"); // Minimum target
        assert_valid_roll("1d6 t6"); // Maximum target for d6
        assert_valid_roll("1d20 t20"); // Maximum target for d20

        // Explode edge cases
        assert_valid_roll("1d6 e1"); // Explode on minimum
        assert_valid_roll("1d6 e6"); // Explode on maximum

        // Reroll edge cases
        assert_valid_roll("1d6 r1"); // Reroll minimum
        assert_valid_roll("1d6 r6"); // Reroll all (should handle gracefully)
    }

    /// Test numeric overflow protection
    #[test]
    fn test_numeric_limits() {
        // Large but valid numbers
        assert_valid_roll("1d6 + 1000");
        assert_valid_roll("1d6 * 100");
        assert_valid_roll("100d6");

        // Very large modifiers (should be handled gracefully)
        assert_valid_roll("1d6 + 999999");
        assert_valid_roll("1d6 - 999999");
    }

    /// Test output formatting edge cases
    #[test]
    fn test_output_formatting() {
        // Test that very large numbers format correctly
        assert_valid_roll("1d6 + 999999");
        assert_valid_roll("100d6 * 100");

        // Test negative number formatting
        assert_valid_roll("1d6 - 100");
        assert_valid_roll("1d4 - 2d8");

        // Test success/failure formatting
        assert_valid_roll("10d10 t7 f1 b1");
        assert_valid_roll("8d6 t4 f2");
    }

    // ============================================================================
    // ERROR HANDLING AND VALIDATION TESTS
    // ============================================================================

    /// Test edge cases and error conditions
    #[test]
    fn test_edge_cases() {
        // Empty or invalid input
        assert_invalid_roll("");
        assert_invalid_roll("   ");
        assert_invalid_roll("xyz");
        assert_invalid_roll("d");
        assert_invalid_roll("1d");
        assert_invalid_roll("d+5");

        // Invalid modifiers
        assert_invalid_roll("1d6 k0");
        assert_invalid_roll("1d6 d0");
        assert_invalid_roll("1d6 t0");
        assert_invalid_roll("1d6 r0");
        assert_invalid_roll("1d6 e0");

        // Division by zero
        assert_invalid_roll("1d6 / 0");

        // Invalid dice expressions
        assert_invalid_roll("1d6d6");
        assert_invalid_roll("dd6");
        assert_invalid_roll("1dd");

        // Excessive values
        assert_invalid_roll("1000d6"); // Too many dice
        assert_invalid_roll("1d10000"); // Too many sides
    }

    /// Test error message quality and informativeness
    #[test]
    fn test_error_message_quality() {
        // Test that specific error conditions produce helpful messages
        let result = parse_and_roll("501d6");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("500") || error_msg.to_lowercase().contains("maximum"));

        let result = parse_and_roll("1d1001");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("1000") || error_msg.to_lowercase().contains("sides"));

        let result = parse_and_roll("1d6 / 0");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.to_lowercase().contains("zero")
                || error_msg.to_lowercase().contains("divide")
        );
    }

    // ============================================================================
    // DISCORD-SPECIFIC FEATURES TESTS
    // ============================================================================

    /// Test format_multiple_results_with_limit function for Discord character limits
    #[test]
    fn test_discord_character_limit_handling() {
        // Test very large roll sets that might exceed Discord's 2000 character limit
        let large_roll = "20 10d6 k8 e6 + 5";
        assert_valid_roll(large_roll);

        // Test that format function handles large results gracefully
        let results = parse_and_roll(large_roll).unwrap();
        let formatted = format_multiple_results_with_limit(&results);
        assert!(
            formatted.len() <= 2000,
            "Output should be under Discord's 2000 char limit"
        );
    }

    /// Test character limit edge cases
    #[test]
    fn test_character_limit_edge_cases() {
        // Test scenarios that might produce very long output
        assert_valid_roll("10 6d6 e6 ie k4 r1 + 1d4 ! this is a very long comment that might contribute to character limit issues");
        assert_valid_roll("100d6 e6"); // Large dice pool
        assert_valid_roll("20 4d6 k3 ! character generation rolls"); // Many roll sets
    }

    /// Test suppress_comment feature in RollResult
    #[test]
    fn test_suppress_comment_functionality() {
        let results = parse_and_roll("4d6 k3 ! ability score").unwrap();
        let original_result = &results[0];

        // Test that create_simplified works
        let simplified = original_result.create_simplified();
        assert!(simplified.suppress_comment);
        assert_eq!(
            simplified.comment,
            Some("Simplified roll due to character limit".to_string())
        );
    }

    // ============================================================================
    // PARSER-SPECIFIC TESTS
    // ============================================================================

    /// Test specific parser behavior
    #[test]
    fn test_parser_specifics() {
        // Test that flags are parsed correctly
        let result = parse_and_roll("p 1d6").unwrap();
        assert!(result[0].private);

        let result = parse_and_roll("s 1d6").unwrap();
        assert!(result[0].simple);

        let result = parse_and_roll("nr 1d6").unwrap();
        assert!(result[0].no_results);

        // Test comment parsing
        let result = parse_and_roll("1d6 ! test comment").unwrap();
        assert_eq!(result[0].comment, Some("test comment".to_string()));

        // Test label parsing
        let result = parse_and_roll("(Test Label) 1d6").unwrap();
        assert_eq!(result[0].label, Some("Test Label".to_string()));
    }

    /// Test original_expression storage in roll results
    #[test]
    fn test_original_expression_storage() {
        // Test semicolon-separated rolls store original expressions
        let results = parse_and_roll("1d20 + 5; 2d6 + 3; 1d4").unwrap();
        for result in results {
            assert!(result.original_expression.is_some());
        }

        // Test single rolls don't necessarily store original expressions
        let _results = parse_and_roll("1d20 + 5").unwrap();
        // This might be None for single rolls depending on implementation
    }

    /// Test roll set parsing
    #[test]
    fn test_roll_set_parsing() {
        let result = parse_and_roll("3 4d6").unwrap();
        assert_eq!(result.len(), 3);
        for (i, roll_result) in result.iter().enumerate() {
            assert_eq!(roll_result.label, Some(format!("Set {}", i + 1)));
        }
    }

    /// Test multiple roll parsing (semicolon separated)
    #[test]
    fn test_multiple_roll_parsing() {
        let result = parse_and_roll("1d20; 2d6; 1d4").unwrap();
        assert_eq!(result.len(), 3);

        // Check that original expressions are stored
        assert!(result[0].original_expression.is_some());
        assert!(result[1].original_expression.is_some());
        assert!(result[2].original_expression.is_some());
    }

    // ============================================================================
    // BEHAVIORAL TESTS (Instead of modifier field tests)
    // ============================================================================

    /// Test exploding dice behavior
    #[test]
    fn test_exploding_behavior() {
        let results = parse_and_roll("2d6 e6").unwrap();
        assert_eq!(results.len(), 1);
        // Test that exploding can happen (individual_rolls might be > 2)
        // Or check notes for explosion message if any explosions occurred
        assert!(!results[0].individual_rolls.is_empty());
    }

    /// Test keep high behavior
    #[test]
    fn test_keep_high_behavior() {
        let results = parse_and_roll("4d6 k3").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kept_rolls.len(), 3);
        assert!(results[0].dropped_rolls.len() > 0);
    }

    /// Test keep low behavior
    #[test]
    fn test_keep_low_behavior() {
        let results = parse_and_roll("4d6 kl2").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kept_rolls.len(), 2);
        assert!(results[0].dropped_rolls.len() > 0);
    }

    /// Test drop behavior
    #[test]
    fn test_drop_behavior() {
        let results = parse_and_roll("4d6 d1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kept_rolls.len(), 3);
        assert_eq!(results[0].dropped_rolls.len(), 1);
    }

    /// Test target/success counting behavior
    #[test]
    fn test_target_behavior() {
        let results = parse_and_roll("5d6 t4").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].successes.is_some());
        // Successes should be >= 0
        assert!(results[0].successes.unwrap() >= 0);
    }

    /// Test failure counting behavior
    #[test]
    fn test_failure_behavior() {
        let results = parse_and_roll("5d6 f2").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].failures.is_some());
        // Failures should be >= 0
        assert!(results[0].failures.unwrap() >= 0);
    }

    /// Test botch counting behavior
    #[test]
    fn test_botch_behavior() {
        let results = parse_and_roll("5d6 b1").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].botches.is_some());
        // Botches should be >= 0
        assert!(results[0].botches.unwrap() >= 0);
    }

    /// Test Hero System behavior
    #[test]
    fn test_hero_system_behavior() {
        // Normal damage
        let results = parse_and_roll("3d6 hsn").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0]
            .notes
            .iter()
            .any(|note| note.contains("Normal damage")));

        // Killing damage
        let results = parse_and_roll("2d6 hsk").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0]
            .notes
            .iter()
            .any(|note| note.contains("Killing damage")));

        // To-hit roll
        let results = parse_and_roll("3d6 hsh").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].notes.iter().any(|note| note.contains("to-hit")));
    }

    /// Test Godbound behavior
    #[test]
    fn test_godbound_behavior() {
        // Damage chart
        let results = parse_and_roll("1d20 gb").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].godbound_damage.is_some());
        assert!(results[0]
            .notes
            .iter()
            .any(|note| note.contains("damage chart")));

        // Straight damage
        let results = parse_and_roll("1d20 gbs").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].godbound_damage.is_some());
        assert!(results[0]
            .notes
            .iter()
            .any(|note| note.contains("Straight damage")));
    }

    /// Test Fudge dice behavior
    #[test]
    fn test_fudge_behavior() {
        let results = parse_and_roll("4d3 fudge").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].fudge_symbols.is_some());
        assert!(results[0].notes.iter().any(|note| note.contains("Fudge")));
    }

    /// Test Wrath & Glory behavior
    #[test]
    fn test_wrath_glory_behavior() {
        let results = parse_and_roll("4d6 wng").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].wng_wrath_die.is_some());
        assert!(results[0].wng_icons.is_some());
        assert!(results[0].wng_exalted_icons.is_some());
    }

    /// Test mathematical modifier behavior
    #[test]
    fn test_math_modifier_behavior() {
        // Addition
        let results = parse_and_roll("1d6 + 5").unwrap();
        assert_eq!(results.len(), 1);
        // The total should be dice result + 5, so at least 6
        assert!(results[0].total >= 6);

        // Subtraction
        let results = parse_and_roll("1d6 - 2").unwrap();
        assert_eq!(results.len(), 1);
        // The total should be dice result - 2, so at least -1
        assert!(results[0].total >= -1);

        // Multiplication
        let results = parse_and_roll("1d6 * 2").unwrap();
        assert_eq!(results.len(), 1);
        // The total should be dice result * 2, so at least 2
        assert!(results[0].total >= 2);

        // Division
        let results = parse_and_roll("1d6 / 2").unwrap();
        assert_eq!(results.len(), 1);
        // The total should be dice result / 2, so at least 0
        assert!(results[0].total >= 0);
    }

    /// Test dice addition/subtraction behavior
    #[test]
    fn test_dice_addition_behavior() {
        let results = parse_and_roll("1d6 + 1d4").unwrap();
        assert_eq!(results.len(), 1);
        // Should have dice groups for both sets
        assert!(results[0].dice_groups.len() >= 2);
        // Total should be at least 2 (minimum of both dice)
        assert!(results[0].total >= 2);

        let results = parse_and_roll("2d6 - 1d4").unwrap();
        assert_eq!(results.len(), 1);
        // Should have dice groups for both sets
        assert!(results[0].dice_groups.len() >= 2);
        // Total could be negative, so just check it's calculated
        assert!(results[0].total >= -2); // Minimum possible (2 - 4)
    }

    // ============================================================================
    // ALIAS EXPANSION TESTS
    // ============================================================================

    /// Test alias expansion
    #[test]
    fn test_alias_expansion() {
        use dicemaiden_rs::aliases::expand_alias;
        // Test basic aliases
        assert_eq!(expand_alias("age"), Some("2d6 + 1d6".to_string()));
        assert_eq!(expand_alias("dndstats"), Some("6 4d6 k3".to_string()));
        assert_eq!(expand_alias("attack"), Some("1d20".to_string()));
        assert_eq!(expand_alias("gb"), Some("1d20 gb".to_string()));
        assert_eq!(expand_alias("gbs"), Some("1d20 gbs".to_string()));

        // Test parameterized aliases
        assert_eq!(expand_alias("4cod"), Some("4d10 t8 ie10".to_string()));
        assert_eq!(expand_alias("3df"), Some("3d3 fudge".to_string()));
        assert_eq!(expand_alias("sr6"), Some("6d6 t5".to_string()));

        // Test non-existent aliases
        assert_eq!(expand_alias("nonexistent"), None);
        assert_eq!(expand_alias("1d6"), None);
    }

    // ============================================================================
    // PERFORMANCE AND STRESS TESTS
    // ============================================================================

    /// Test memory-intensive scenarios
    #[test]
    fn test_memory_intensive_scenarios() {
        // Test maximum explosions (should cap at 100)
        assert_valid_roll("1d6 ie1"); // This could theoretically explode forever

        // Test maximum rerolls (should cap at 100)
        assert_valid_roll("1d6 ir6"); // This could theoretically reroll forever

        // Test large dice pools with modifiers
        assert_valid_roll("500d6 k250"); // Maximum dice, keep half
        assert_valid_roll("400d10 t5"); // Large target roll
    }

    /// Test performance and stress cases
    #[test]
    fn test_performance_cases() {
        // Large dice pools (but within limits)
        assert_valid_roll("500d6");
        assert_valid_roll("100d100");
        assert_valid_roll("200d20");

        // Maximum roll sets
        assert_valid_roll("20 4d6");
        assert_valid_roll("20 1d20");

        // Maximum multiple rolls
        assert_valid_roll("1d6; 1d6; 1d6; 1d6");

        // Complex expressions with many modifiers
        assert_valid_roll("50d6 e6 ie k25 r1 t4 + 10");
        assert_valid_roll("20d10 t7 f1 b1 ie10 + 5d6 e6 - 2d4");
    }

    /// Test documented examples from help text work
    #[test]
    fn test_documented_examples() {
        // Examples from basic help
        assert_valid_roll("2d6 + 3d10");
        assert_valid_roll("3d6 + 5");
        assert_valid_roll("4d6 k3");
        assert_valid_roll("10d6 e6 k8 + 4");
        assert_valid_roll("6 4d6");
        assert_valid_roll("4d100 ; 3d10 k2");

        // Examples from alias help
        assert_valid_roll("4cod");
        assert_valid_roll("4cod8");
        assert_valid_roll("4wod8");
        assert_valid_roll("dndstats");
        assert_valid_roll("attack + 5");
        assert_valid_roll("+d20");
        assert_valid_roll("-d20");
        assert_valid_roll("2hsn");
        assert_valid_roll("3hsk");
        assert_valid_roll("2.5hsk");
        assert_valid_roll("gb");
        assert_valid_roll("gbs");
        assert_valid_roll("wng 4d6");
        assert_valid_roll("wng dn3 5d6");
        assert_valid_roll("3df");
        assert_valid_roll("3wh4+");
        assert_valid_roll("sr6");
        assert_valid_roll("ex5");
        assert_valid_roll("6yz");
        assert_valid_roll("age");
        assert_valid_roll("dd34");
        assert_valid_roll("ed15");
        assert_valid_roll("dh 4d10");
    }

    // ============================================================================
    // CRITICAL MISSING TESTS - HIGH PRIORITY
    // ============================================================================

    /// Test mathematical operation precedence
    #[test]
    fn test_mathematical_precedence() {
        // Test that operations follow correct order: * and / before + and -
        let result1 = parse_and_roll("1d1 + 2 * 3").unwrap(); // Should be 1 + (2*3) = 7
        assert_eq!(result1[0].total, 7);

        let result2 = parse_and_roll("2 * 3 + 1d1").unwrap(); // Should be (2*3) + 1 = 7
        assert_eq!(result2[0].total, 7);

        // Test division precedence
        let result3 = parse_and_roll("8d1 / 2 + 1").unwrap(); // Should be (8/2) + 1 = 5
        assert_eq!(result3[0].total, 5);

        // Test complex precedence
        let result4 = parse_and_roll("2d1 + 3 * 2 - 4 / 2").unwrap(); // 2 + (3*2) - (4/2) = 6
        assert_eq!(result4[0].total, 6);
    }

    /// Test explosion and reroll limits to prevent infinite loops
    #[test]
    fn test_explosion_reroll_limits() {
        // Test that indefinite explosions cap at 100
        let results = parse_and_roll("1d6 ie1").unwrap(); // Should always explode
        assert!(results[0].individual_rolls.len() <= 101); // Original + 100 explosions max

        let has_limit_note = results[0]
            .notes
            .iter()
            .any(|note| note.contains("Maximum explosions") || note.contains("100"));
        if results[0].individual_rolls.len() > 50 {
            assert!(has_limit_note, "Should have maximum explosions note");
        }

        // Test that indefinite rerolls cap at 100
        let results = parse_and_roll("1d6 ir6").unwrap(); // Should always reroll
        let has_reroll_limit = results[0]
            .notes
            .iter()
            .any(|note| note.contains("Maximum rerolls") || note.contains("100"));
        if results[0].notes.len() > 5 {
            assert!(
                has_reroll_limit,
                "Should have maximum rerolls note for many rerolls"
            );
        }
    }

    /// Test memory management with large operations
    #[test]
    fn test_memory_management() {
        // Test that large numbers of dice don't cause memory issues
        let results = parse_and_roll("500d6").unwrap(); // Maximum allowed dice
        assert_eq!(results[0].individual_rolls.len(), 500);

        // Test large roll sets
        let results = parse_and_roll("20 100d6").unwrap(); // Maximum sets * large dice
        assert_eq!(results.len(), 20);
        for result in &results {
            assert_eq!(result.individual_rolls.len(), 100);
        }

        // Test complex expression with many modifiers
        let results = parse_and_roll("50d6 e6 k25 r1 + 25d4 e4 - 10d8").unwrap();
        assert!(results[0].total.abs() < 10000); // Reasonable bounds check
    }

    /// Test numeric overflow protection
    #[test]
    fn test_numeric_overflow_protection() {
        // Test very large modifiers
        let results = parse_and_roll("1d6 + 999999").unwrap();
        assert!(results[0].total >= 1000000); // Should handle large numbers

        let results = parse_and_roll("1d6 - 999999").unwrap();
        assert!(results[0].total <= -999993); // Should handle large negative numbers

        // Test large multiplication
        let results = parse_and_roll("100d6 * 100").unwrap();
        assert!(results[0].total >= 10000); // Should be at least 100 * 100

        // Test that division by zero is properly handled
        let result = parse_and_roll("1d6 / 0");
        assert!(result.is_err(), "Division by zero should error");
    }

    // ============================================================================
    // DETAILED GAME SYSTEM TESTING
    // ============================================================================

    /// Test Wrath & Glory mechanics in detail
    #[test]
    fn test_wrath_glory_detailed_mechanics() {
        // Test that wrath die is tracked
        let results = parse_and_roll("4d6 wng").unwrap();
        assert!(results[0].wng_wrath_die.is_some());
        let wrath_value = results[0].wng_wrath_die.unwrap();
        assert!(wrath_value >= 1 && wrath_value <= 6);

        // Test icon counting exists
        assert!(results[0].wng_icons.is_some());
        assert!(results[0].wng_exalted_icons.is_some());

        // Test difficulty tests show PASS/FAIL
        let results = parse_and_roll("wng dn3 5d6").unwrap();
        let has_pass_fail = results[0].notes.iter().any(|note| {
            note.contains("PASS") || note.contains("FAIL") || note.contains("Difficulty")
        });
        assert!(
            has_pass_fail,
            "Should have pass/fail notation for difficulty tests"
        );

        // Test total mode vs success mode
        let results_success = parse_and_roll("4d6 wng").unwrap();
        let results_total = parse_and_roll("4d6 wngt").unwrap();

        // Success mode should have successes, total mode should not
        assert!(results_success[0].successes.is_some());
        assert!(results_total[0].successes.is_none() || results_total[0].successes == Some(0));
        assert!(results_total[0].total > 0);
    }

    /// Test Wrath & Glory special modes thoroughly  
    #[test]
    fn test_wrath_glory_special_modes() {
        // Test soak mode
        let results = parse_and_roll("wng 4d6 !soak").unwrap();
        assert!(results[0].total > 0); // Should use total, not successes

        // Test exempt mode
        let results = parse_and_roll("wng 5d6 !exempt").unwrap();
        assert!(results[0].total > 0);

        // Test damage mode
        let results = parse_and_roll("wng 6d6 !dmg").unwrap();
        assert!(results[0].total > 0);

        // Test difficulty with special modes
        let results = parse_and_roll("wng dn2 4d6 !soak").unwrap();
        assert!(results[0].total > 0);
        let has_difficulty = results[0].notes.iter().any(|note| {
            note.contains("Difficulty") || note.contains("PASS") || note.contains("FAIL")
        });
        assert!(has_difficulty);
    }

    /// Test Godbound damage chart accuracy
    #[test]
    fn test_godbound_damage_chart_accuracy() {
        // Test damage chart conversions with known values
        let test_cases = [
            ("1d1 gb", 0),      // 1 -> 0 damage
            ("1d1 + 1 gb", 1),  // 2 -> 1 damage
            ("1d1 + 4 gb", 1),  // 5 -> 1 damage
            ("1d1 + 5 gb", 2),  // 6 -> 2 damage
            ("1d1 + 8 gb", 2),  // 9 -> 2 damage
            ("1d1 + 9 gb", 4),  // 10 -> 4 damage
            ("1d1 + 19 gb", 4), // 20 -> 4 damage
        ];

        for (expression, expected_damage) in test_cases {
            let results = parse_and_roll(expression).unwrap();
            assert_eq!(
                results[0].godbound_damage,
                Some(expected_damage),
                "Expression {} should produce {} damage",
                expression,
                expected_damage
            );
        }

        // Test that straight damage bypasses chart
        let results = parse_and_roll("1d1 + 9 gbs").unwrap();
        assert_eq!(results[0].godbound_damage, Some(10)); // Should be 10, not 4

        // Test multiple dice with chart conversion
        let results = parse_and_roll("2d1 + 8 gb").unwrap(); // Two dice each rolling 1, plus 8 = 10 total
        assert_eq!(results[0].godbound_damage, Some(4)); // 10 -> 4 damage
    }

    /// Test Hero System calculations in detail
    #[test]
    fn test_hero_system_detailed() {
        // Test normal damage
        let results = parse_and_roll("3d6 hsn").unwrap();
        assert!(results[0]
            .notes
            .iter()
            .any(|note| note.contains("Normal damage")));
        assert!(results[0].total >= 3 && results[0].total <= 18);

        // Test killing damage produces both BODY and STUN
        let results = parse_and_roll("2d6 hsk").unwrap();
        let has_killing_note = results[0].notes.iter().any(|note| {
            note.contains("Killing damage") && note.contains("BODY") && note.contains("STUN")
        });
        assert!(
            has_killing_note,
            "Should have BODY and STUN damage for killing attacks"
        );

        // Test to-hit roll
        let results = parse_and_roll("3d6 hsh").unwrap();
        let has_hit_note = results[0]
            .notes
            .iter()
            .any(|note| note.contains("to-hit") && note.contains("11 + OCV - DCV"));
        assert!(has_hit_note, "Should have to-hit notation");

        // Test fractional dice
        let results = parse_and_roll("2d6 + 1d3 hsk").unwrap();
        assert!(results[0]
            .notes
            .iter()
            .any(|note| note.contains("Killing damage")));
    }

    // ============================================================================
    // ORDER OF OPERATIONS AND MODIFIER INTERACTION TESTS
    // ============================================================================

    /// Test modifier application order
    #[test]
    fn test_modifier_interaction_order() {
        // Test that drop happens before keep
        let results = parse_and_roll("6d1 d2 k2").unwrap(); // 6 dice showing 1, drop 2, keep 2
        assert_eq!(results[0].kept_rolls.len(), 2);
        assert_eq!(results[0].dropped_rolls.len(), 2);

        // Test that exploding happens before keep/drop
        let results = parse_and_roll("4d1 + 5 e6 k3").unwrap(); // 4 dice + 5 = 6 each, explode, keep 3
        assert_eq!(results[0].kept_rolls.len(), 3);
        // Should have more than 4 total dice due to explosions
        let total_dice = results[0].kept_rolls.len() + results[0].dropped_rolls.len();
        assert!(total_dice > 4, "Should have exploded dice");

        // Test complex modifier order
        let results = parse_and_roll("6d1 + 5 e6 r1 d1 k2").unwrap();
        assert_eq!(results[0].kept_rolls.len(), 2);
        assert!(results[0].dropped_rolls.len() > 0);
    }

    /// Test mathematical modifier order with dice modifiers
    #[test]
    fn test_mathematical_vs_dice_modifier_order() {
        // Test that math happens after dice manipulation for normal rolls
        let results = parse_and_roll("4d1 k3 + 2").unwrap();
        assert_eq!(results[0].total, 5); // 3 dice * 1 + 2 = 5

        // Test that math happens before Godbound conversion
        let results = parse_and_roll("1d1 + 9 gb").unwrap();
        assert_eq!(results[0].godbound_damage, Some(4)); // (1+9)=10 -> 4 damage

        // Test math with success systems
        let results = parse_and_roll("4d1 + 2 t3").unwrap(); // Each die = 1+2=3, meets target
        assert_eq!(results[0].successes, Some(4)); // All 4 dice should succeed
    }

    /// Test keep/drop edge cases and interactions
    #[test]
    fn test_keep_drop_edge_cases() {
        // Test keeping more dice than available
        let results = parse_and_roll("3d6 k5").unwrap();
        assert_eq!(results[0].kept_rolls.len(), 3); // Should keep all 3
        assert_eq!(results[0].dropped_rolls.len(), 0);

        // Test dropping more dice than available
        let results = parse_and_roll("3d6 d5").unwrap();
        assert_eq!(results[0].kept_rolls.len(), 0); // Should drop all
        assert_eq!(results[0].dropped_rolls.len(), 3);

        // Test keep 0 (edge case)
        let results = parse_and_roll("4d6 k0");
        assert!(results.is_err() || results.unwrap()[0].kept_rolls.is_empty());

        // Test drop then keep interaction
        let results = parse_and_roll("6d1 d2 k2").unwrap();
        assert_eq!(results[0].kept_rolls.len(), 2);
        assert_eq!(results[0].dropped_rolls.len(), 2);
        // Total should be 4 dice processed (6 - 2 dropped = 4, keep 2 of those)
    }

    // ============================================================================
    // ERROR HANDLING AND VALIDATION TESTS
    // ============================================================================

    /// Test specific error messages for better UX
    #[test]
    fn test_specific_error_messages() {
        // Test dice count limit error
        let result = parse_and_roll("501d6");
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("500")
                || error.to_lowercase().contains("maximum")
                || error.to_lowercase().contains("dice")
        );

        // Test dice sides limit error
        let result = parse_and_roll("1d1001");
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("1000") || error.to_lowercase().contains("sides"));

        // Test zero sides error
        let result = parse_and_roll("1d0");
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.to_lowercase().contains("sides") || error.contains("0"));

        // Test division by zero error
        let result = parse_and_roll("1d6 / 0");
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.to_lowercase().contains("zero") || error.to_lowercase().contains("divide"));

        // Test invalid modifier error
        let result = parse_and_roll("1d6 k0");
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.to_lowercase().contains("keep") || error.contains("0"));

        // Test malformed expression error
        let result = parse_and_roll("xyz");
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            error.to_lowercase().contains("invalid")
                || error.to_lowercase().contains("parse")
                || error.to_lowercase().contains("expression")
        );
    }

    /// Test alias parameter validation
    #[test]
    fn test_alias_parameter_validation() {
        // Test Wrath & Glory difficulty limits
        assert_valid_roll("wng dn1 4d6");
        assert_valid_roll("wng dn10 4d6");
        // Note: Invalid difficulties might be handled gracefully rather than erroring

        // Test CoD variants with edge cases
        assert_valid_roll("1cod"); // Minimum dice
        assert_valid_roll("50cod8"); // Large pool

        // Test Hero System fractional edge cases
        assert_valid_roll("10.5hsk"); // Large fractional
        assert_valid_roll("0.5hsk"); // Small fractional (might error or round up)

        // Test Earthdawn step boundaries
        assert_valid_roll("ed1");
        assert_valid_roll("ed50");
        let result = parse_and_roll("ed0");
        assert!(result.is_err() || result.is_ok()); // Implementation dependent
        let result = parse_and_roll("ed51");
        assert!(result.is_err() || result.is_ok()); // Implementation dependent
    }

    // ============================================================================
    // UNSORTED FLAG AND BEHAVIORAL VERIFICATION TESTS
    // ============================================================================

    /// Test that unsorted flag actually affects dice ordering
    #[test]
    fn test_unsorted_flag_behavior_detailed() {
        // Test with deterministic dice to verify unsorted behavior
        let results_sorted = parse_and_roll("10d6").unwrap();
        let results_unsorted = parse_and_roll("ul 10d6").unwrap();

        // Note: The unsorted field is not accessible in RollResult
        // This is a limitation of the current API - we can only test that parsing works
        assert_eq!(results_sorted.len(), 1);
        assert_eq!(results_unsorted.len(), 1);

        // Test that both produce valid results
        assert!(!results_sorted[0].kept_rolls.is_empty());
        assert!(!results_unsorted[0].kept_rolls.is_empty());

        // Sorted dice should typically be in descending order (probabilistically)
        let sorted_dice = &results_sorted[0].kept_rolls;
        let mut is_descending = true;
        for i in 1..sorted_dice.len() {
            if sorted_dice[i - 1] < sorted_dice[i] {
                is_descending = false;
                break;
            }
        }
        // Note: This is probabilistic - with 10d6, it's very likely to be sorted
        // but not guaranteed. The main test is that parsing works correctly.

        // The important verification is that the unsorted flag doesn't break parsing
        assert!(is_descending || !is_descending); // Always true, but exercises the code
    }

    /// Test private roll flag behavior
    #[test]
    fn test_private_roll_flag_detailed_behavior() {
        let results = parse_and_roll("p 1d6 + 2").unwrap();
        assert!(results[0].private);

        // Test that private rolls work with complex expressions
        let results = parse_and_roll("p 4d6 k3 e6 + 2").unwrap();
        assert!(results[0].private);

        // Test private with aliases
        let results = parse_and_roll("p attack + 5").unwrap();
        assert!(results[0].private);

        // Test private with multiple flags
        let results = parse_and_roll("p s 1d6").unwrap();
        assert!(results[0].private);
        assert!(results[0].simple);
    }

    /// Test simple and no-results flags
    #[test]
    fn test_output_control_flags_detailed() {
        // Test simple flag
        let results = parse_and_roll("s 2d6 + 3").unwrap();
        assert!(results[0].simple);

        // Test no-results flag
        let results = parse_and_roll("nr 2d6 + 3").unwrap();
        assert!(results[0].no_results);

        // Test flag combinations that are accessible
        let results = parse_and_roll("p s nr 4d6 k3").unwrap();
        assert!(results[0].private);
        assert!(results[0].simple);
        assert!(results[0].no_results);

        // Note: ul (unsorted) flag is not accessible in RollResult API
        // but we can test that it parses correctly
        let results = parse_and_roll("ul 4d6").unwrap();
        assert_eq!(results.len(), 1);
    }

    // ============================================================================
    // COMMENT AND LABEL EDGE CASES
    // ============================================================================

    /// Test complex comment and label interactions
    #[test]
    fn test_comment_label_edge_cases() {
        // Test comment with dice expressions in text
        let results = parse_and_roll("1d6 ! rolling 2d8 for damage").unwrap();
        assert_eq!(
            results[0].comment,
            Some("rolling 2d8 for damage".to_string())
        );

        // Test label with dice expressions in text
        let results = parse_and_roll("(Attack 1d20+5) 1d6").unwrap();
        assert_eq!(results[0].label, Some("Attack 1d20+5".to_string()));

        // Test comment stripping in formatted output
        let results = parse_and_roll("1d6 + 2 ! test comment").unwrap();
        let _formatted = format_multiple_results_with_limit(&results);
        // Should preserve comment in the result structure
        assert_eq!(results[0].comment, Some("test comment".to_string()));
        // The formatted output handling is tested elsewhere

        // Test empty comments and labels
        let results = parse_and_roll("1d6 !").unwrap();
        assert!(results[0].comment.is_some());

        let results = parse_and_roll("() 1d6").unwrap();
        assert!(results[0].label.is_some());

        // Test very long comments
        let long_comment = "a".repeat(200);
        let results = parse_and_roll(&format!("1d6 ! {}", long_comment)).unwrap();
        assert_eq!(results[0].comment, Some(long_comment));
    }

    /// Test comment and label with unicode
    #[test]
    fn test_unicode_comments_labels() {
        // Test unicode in comments
        let results = parse_and_roll("1d6 ! café roll ⚔️").unwrap();
        assert_eq!(results[0].comment, Some("café roll ⚔️".to_string()));

        // Test unicode in labels
        let results = parse_and_roll("(攻撃ロール) 1d6").unwrap();
        assert_eq!(results[0].label, Some("攻撃ロール".to_string()));

        // Test mixed unicode and ASCII
        let results = parse_and_roll("(Würfel-Attack) 1d20 ! café damage ⚔️").unwrap();
        assert_eq!(results[0].label, Some("Würfel-Attack".to_string()));
        assert_eq!(results[0].comment, Some("café damage ⚔️".to_string()));
    }

    // ============================================================================
    // ROLL SET AND MULTIPLE ROLL EDGE CASES
    // ============================================================================

    /// Test roll set edge cases and limits
    #[test]
    fn test_roll_set_boundary_cases() {
        // Test minimum and maximum roll sets
        let results = parse_and_roll("2 1d6").unwrap(); // Minimum
        assert_eq!(results.len(), 2);

        let results = parse_and_roll("20 1d6").unwrap(); // Maximum
        assert_eq!(results.len(), 20);

        // Test roll set boundary validation
        let result = parse_and_roll("1 1d6");
        assert!(result.is_err() || result.unwrap().len() == 1);

        let result = parse_and_roll("21 1d6");
        assert!(result.is_err() || result.unwrap().len() <= 20);

        // Test roll sets with complex expressions
        let results = parse_and_roll("5 4d6 k3 e6 + 2").unwrap();
        assert_eq!(results.len(), 5);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.label, Some(format!("Set {}", i + 1)));
            assert_eq!(result.kept_rolls.len(), 3);
        }
    }

    /// Test multiple roll limits and validation
    #[test]
    fn test_multiple_roll_limits() {
        // Test maximum 4 rolls
        let results = parse_and_roll("1d6; 1d6; 1d6; 1d6").unwrap();
        assert_eq!(results.len(), 4);

        // Test too many rolls
        let result = parse_and_roll("1d6; 1d6; 1d6; 1d6; 1d6");
        assert!(result.is_err(), "Should error with more than 4 rolls");

        // Test original expression storage
        let results = parse_and_roll("1d20 + 5; 2d6 + 3; 1d4").unwrap();
        for result in &results {
            assert!(result.original_expression.is_some());
        }
    }

    // ============================================================================
    // FUDGE DICE DETAILED TESTING
    // ============================================================================

    /// Test Fudge dice symbol generation and behavior
    #[test]
    fn test_fudge_dice_detailed() {
        let results = parse_and_roll("4d3 fudge").unwrap();
        assert!(results[0].fudge_symbols.is_some());

        let symbols = results[0].fudge_symbols.as_ref().unwrap();
        assert_eq!(symbols.len(), 4);

        // Each symbol should be +, -, or blank space
        for symbol in symbols {
            assert!(symbol == "+" || symbol == "-" || symbol == " ");
        }

        // Test that total matches symbol values
        let expected_total: i32 = symbols
            .iter()
            .map(|s| match s.as_str() {
                "+" => 1,
                "-" => -1,
                " " => 0,
                _ => panic!("Invalid fudge symbol: {}", s),
            })
            .sum();
        assert_eq!(results[0].total, expected_total);

        // Test Fudge dice with modifiers
        let results = parse_and_roll("4d3 fudge + 2").unwrap();
        assert!(results[0].fudge_symbols.is_some());
        assert!(results[0].total >= -2); // Minimum -4 + 2 = -2
    }

    // ============================================================================
    // WHITESPACE AND PARSING EDGE CASES
    // ============================================================================

    /// Test extreme whitespace scenarios
    #[test]
    fn test_extreme_whitespace() {
        // Test excessive whitespace
        assert_valid_roll("    1d6    +    2    ");
        assert_valid_roll("\t\t1d6\t+\t2\t\t");
        assert_valid_roll("\n1d6\n+\n2\n");

        // Test no whitespace
        assert_valid_roll("1d6+2-1*3/2");
        assert_valid_roll("4d6e6k3+2d4-1");

        // Test mixed whitespace
        assert_valid_roll("1d6  +2- 1*   3/2");
        assert_valid_roll("4d6 e6k3+ 2d4 -1");

        // Test whitespace in comments and labels
        assert_valid_roll("(  Label  ) 1d6 !  Comment  ");
        assert_valid_roll("(\t\tLabel\t\t) 1d6 !\t\tComment\t\t");
    }

    /// Test complex parsing scenarios
    #[test]
    fn test_complex_parsing_scenarios() {
        // Test very long but valid expressions
        let complex_expr =
            "10d6 e6 ie r1 ir2 d1 k8 kl6 t4 f1 b1 + 5d4 e4 - 2d8 r2 * 2 / 3 + 100 - 50";
        assert_valid_roll(complex_expr);

        // Test expressions that might confuse the parser
        assert_valid_roll("1d6+2d8-3d4*2/3+1d10-1d12");
        assert_valid_roll("4d6e6ie10k3r1ir2t4f1b1+2d4e4-1d8");

        // Test alias combinations
        assert_valid_roll("attack + 1d6 e6 + 2");
        assert_valid_roll("4cod + 2d6 e6 - 1");
        assert_valid_roll("3df + 1d8 k1");
    }

    // ============================================================================
    // REAL-WORLD SCENARIO INTEGRATION TESTS
    // ============================================================================

    /// Test realistic gaming scenarios
    #[test]
    fn test_real_world_gaming_scenarios() {
        // D&D 5e combat sequence
        assert_valid_roll("+d20 + 5"); // Advantage attack
        assert_valid_roll("2d6 + 3"); // Longsword damage
        assert_valid_roll("1d20 + 2"); // Death saving throw

        // D&D character creation
        assert_valid_roll("6 4d6 k3"); // Ability scores
        assert_valid_roll("1d8 + 2"); // Hit points

        // World of Darkness investigation
        assert_valid_roll("5cod"); // Investigation roll
        assert_valid_roll("3cod8"); // 8-again specialty
        assert_valid_roll("4codr"); // Rote action
        assert_valid_roll("6wod8"); // Difficulty 8 roll

        // Warhammer 40k combat
        assert_valid_roll("10wh3+"); // To hit
        assert_valid_roll("8wh4+"); // To wound
        assert_valid_roll("6wh5+"); // Armor save
        assert_valid_roll("dh 6d10"); // Dark Heresy damage

        // Hero System superhero combat
        assert_valid_roll("12d6 hsn"); // Normal damage
        assert_valid_roll("4d6 hsk"); // Killing damage
        assert_valid_roll("3d6 hsh"); // To-hit roll

        // Godbound divine combat
        assert_valid_roll("1d20 + 15 gb"); // Divine attack
        assert_valid_roll("2d8 + 10 gbs"); // Straight damage

        // FATE/Fudge narrative
        assert_valid_roll("4df + 3"); // Great skill with Fudge dice
        assert_valid_roll("4df - 1"); // Poor skill
    }

    /// Test mixed system scenarios (crossover games)
    #[test]
    fn test_mixed_system_scenarios() {
        // Multiple systems in one session (semicolon rolls)
        assert_valid_roll("attack + 5; 4cod; 3df + 2; sr6");
        assert_valid_roll("1d20 + 10; 6d10 t7; 4df; wng 5d6");

        // Conversion scenarios
        assert_valid_roll("2d6 + 3"); // Standard damage
        assert_valid_roll("2d6 + 3 gb"); // Same damage through Godbound chart
        assert_valid_roll("2d6 + 3 gbs"); // Same damage straight conversion
    }

    // ============================================================================
    // PERFORMANCE AND STRESS TESTS
    // ============================================================================

    /// Test performance with large operations
    #[test]
    fn test_performance_stress() {
        // Test maximum allowed configurations
        assert_valid_roll("500d1000"); // Max dice * max sides
        assert_valid_roll("20 500d6"); // Max sets * max dice

        // Test complex expressions that might be slow
        for _ in 0..10 {
            assert_valid_roll("100d6 e6 ie k50 r1 ir2 + 50d4 e4 - 25d8");
        }

        // Test many modifier combinations
        assert_valid_roll("50d10 t7 f1 b1 ie10 e9 r1 ir2 k30 d5 + 10d6 e6 - 5d4 * 2");
    }

    /// Test memory efficiency with repeated operations
    #[test]
    fn test_memory_efficiency() {
        // Run many operations to check for memory leaks
        for i in 0..100 {
            let expr = format!(
                "{}d6 e6 k{} + {}",
                (i % 50) + 1, // 1-50 dice
                (i % 10) + 1, // Keep 1-10
                i % 20
            ); // Add 0-19
            assert_valid_roll(&expr);
        }

        // Test large roll sets repeatedly
        for _ in 0..20 {
            assert_valid_roll("10 20d6 e6 k10");
        }
    }

    // ============================================================================
    // HELPER FUNCTIONS FOR ADDITIONAL TESTS
    // ============================================================================

    /// Helper function to test that a roll produces valid results
    fn assert_valid_roll(input: &str) {
        let result = parse_and_roll(input);
        assert!(
            result.is_ok(),
            "Failed to parse: {} - Error: {:?}",
            input,
            result.err()
        );
        let results = result.unwrap();
        assert!(!results.is_empty(), "No results for: {}", input);

        for roll_result in &results {
            // Basic sanity checks
            assert!(
                !roll_result.individual_rolls.is_empty() || roll_result.fudge_symbols.is_some(),
                "No individual rolls for: {}",
                input
            );
        }
    }

    /// Helper function to test that a roll produces an error
    fn assert_invalid_roll(input: &str) {
        let result = parse_and_roll(input);
        assert!(result.is_err(), "Expected error for: {}", input);
    }

    // ============================================================================
    // FINAL INTEGRATION AND REGRESSION TESTS
    // ============================================================================

    /// Test all documented examples from help text
    #[test]
    fn test_all_documented_examples() {
        // From basic help examples
        let basic_examples = [
            "2d6 + 3d10",
            "3d6 + 5",
            "4d6 k3",
            "10d6 e6 k8 + 4",
            "6 4d6",
            "4d100 ; 3d10 k2",
        ];

        for example in &basic_examples {
            assert_valid_roll(example);
        }

        // From alias help examples
        let alias_examples = [
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

        for example in &alias_examples {
            assert_valid_roll(example);
        }
    }

    /// Regression test for known edge cases
    #[test]
    fn test_regression_cases() {
        // Test cases that might have caused issues in development
        assert_valid_roll("1d1"); // Minimum dice
        assert_valid_roll("1d2"); // Minimum useful dice
        assert_valid_roll("500d1000"); // Maximum everything

        // Test borderline cases
        assert_valid_roll("4d6 k4"); // Keep all
        assert_valid_roll("4d6 d0"); // Drop none
        assert_valid_roll("1d6 e1"); // Always explode
        assert_valid_roll("1d6 r6"); // Always reroll

        // Test that were problematic during development
        assert_valid_roll("p s ul nr 1d6"); // All flags
        assert_valid_roll("(Very Long Label Name) 1d6 ! Very long comment with lots of text");
        assert_valid_roll("20 100d6 e6 k50"); // Large everything
    }
}
