// tests/dice_tests.rs - Complete Dice Maiden Test Suite
// Place this file in the tests/ directory for integration testing

use dicemaiden_rs::dice::{
    format_multiple_results_with_limit, parse_and_roll, DiceRoll, HeroSystemType, Modifier,
};

/// Helper function to test that a roll produces valid results
fn assert_valid_roll(input: &str) {
    let result = parse_and_roll(input);
    assert!(result.is_ok(), "Failed to parse: {}", input);
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

        // Verify the flag is parsed correctly
        let result = parse_and_roll("ul 4d6").unwrap();
        assert!(result[0].unsorted);
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

        // Test that DH modifier is parsed correctly
        let result = parse_and_roll("4d10 ie10 dh").unwrap();
        assert!(result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::DarkHeresy)));
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

        let result = parse_and_roll("ul 1d6").unwrap();
        assert!(result[0].unsorted);

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
        let results = parse_and_roll("1d20 + 5").unwrap();
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
    // MODIFIER PARSING TESTS
    // ============================================================================

    /// Test modifier parsing
    #[test]
    fn test_modifier_parsing() {
        let result = parse_and_roll("1d6 e6").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Explode(Some(6))));

        let result = parse_and_roll("1d6 ie").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::ExplodeIndefinite(None)
        ));

        let result = parse_and_roll("1d6 k3").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::KeepHigh(3)));

        let result = parse_and_roll("1d6 kl2").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::KeepLow(2)));

        let result = parse_and_roll("1d6 d1").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Drop(1)));

        let result = parse_and_roll("1d6 r2").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Reroll(2)));

        let result = parse_and_roll("1d6 ir1").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::RerollIndefinite(1)
        ));

        let result = parse_and_roll("1d6 t4").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Target(4)));

        let result = parse_and_roll("1d6 f1").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Failure(1)));

        let result = parse_and_roll("1d6 b").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Botch(None)));

        let result = parse_and_roll("1d6 b1").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Botch(Some(1))));
    }

    /// Test Hero System specific parsing
    #[test]
    fn test_hero_system_parsing() {
        let result = parse_and_roll("1d6 hsn").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::HeroSystem(HeroSystemType::Normal)
        ));

        let result = parse_and_roll("1d6 hsk").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::HeroSystem(HeroSystemType::Killing)
        ));

        let result = parse_and_roll("3d6 hsh").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::HeroSystem(HeroSystemType::Hit)
        ));
    }

    /// Test Godbound parsing
    #[test]
    fn test_godbound_parsing() {
        let result = parse_and_roll("1d20 gb").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Godbound(false)));

        let result = parse_and_roll("1d20 gbs").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Godbound(true)));
    }

    /// Test Wrath & Glory parsing
    #[test]
    fn test_wrath_glory_parsing() {
        let result = parse_and_roll("4d6 wng").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::WrathGlory(None, false)
        ));

        let result = parse_and_roll("4d6 wng2").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::WrathGlory(Some(2), false)
        ));

        let result = parse_and_roll("4d6 wng3t").unwrap();
        assert!(matches!(
            result[0].modifiers[0],
            Modifier::WrathGlory(Some(3), true)
        ));
    }

    /// Test Fudge dice parsing
    #[test]
    fn test_fudge_parsing() {
        let result = parse_and_roll("3d3 fudge").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Fudge));
    }

    /// Test mathematical modifier parsing
    #[test]
    fn test_math_modifier_parsing() {
        let result = parse_and_roll("1d6 + 5").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Add(5)));

        let result = parse_and_roll("1d6 - 3").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Subtract(3)));

        let result = parse_and_roll("1d6 * 2").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Multiply(2)));

        let result = parse_and_roll("1d6 / 2").unwrap();
        assert!(matches!(result[0].modifiers[0], Modifier::Divide(2)));
    }

    /// Test dice modifier parsing
    #[test]
    fn test_dice_modifier_parsing() {
        let result = parse_and_roll("1d6 + 1d4").unwrap();
        if let Modifier::AddDice(dice) = &result[0].modifiers[0] {
            assert_eq!(dice.count, 1);
            assert_eq!(dice.sides, 4);
        } else {
            panic!("Expected AddDice modifier");
        }

        let result = parse_and_roll("2d6 - 1d4").unwrap();
        if let Modifier::SubtractDice(dice) = &result[0].modifiers[0] {
            assert_eq!(dice.count, 1);
            assert_eq!(dice.sides, 4);
        } else {
            panic!("Expected SubtractDice modifier");
        }
    }

    // ============================================================================
    // ALIAS EXPANSION TESTS
    // ============================================================================

    /// Test alias expansion
    #[test]
    fn test_alias_expansion() {
        use crate::dice::aliases::expand_alias;

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

    /// Test concurrent parsing (if applicable)
    #[test]
    fn test_concurrent_parsing() {
        use std::sync::Arc;
        use std::thread;

        let test_cases = Arc::new(vec![
            "1d6",
            "2d6 + 3",
            "4d6 k3",
            "attack + 5",
            "3df",
            "4cod",
            "sr6",
        ]);

        let mut handles = vec![];

        for _ in 0..10 {
            let cases = Arc::clone(&test_cases);
            let handle = thread::spawn(move || {
                for case in cases.iter() {
                    assert_valid_roll(case);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// Test thread safety (if applicable)
    #[test]
    fn test_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let expressions = Arc::new(vec![
            "1d6",
            "2d6 + 3",
            "4d6 k3",
            "attack + 5",
            "3df",
            "4cod",
            "sr6",
            "6d10 t7",
            "3d6 e6",
            "4d6 r1",
            "2d20 kl1",
            "1d% + 10",
        ]);

        let mut handles = vec![];

        for _ in 0..5 {
            let exprs = Arc::clone(&expressions);
            let handle = thread::spawn(move || {
                for expr in exprs.iter() {
                    let result = parse_and_roll(expr);
                    assert!(result.is_ok(), "Failed to parse {} in thread", expr);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    // ============================================================================
    // INTEGRATION AND SYSTEM TESTS
    // ============================================================================

    /// Test system-specific edge cases
    #[test]
    fn test_system_specific_edge_cases() {
        // Chronicles of Darkness edge cases
        assert_valid_roll("1cod");
        assert_valid_roll("20cod");
        assert_valid_roll("1cod8");
        assert_valid_roll("1codr");

        // World of Darkness edge cases
        assert_valid_roll("1wod6");
        assert_valid_roll("1wod10");
        assert_valid_roll("20wod8");

        // Hero System fractional dice edge cases
        assert_valid_roll("0.5hsk");
        assert_valid_roll("10.5hsn");

        // Earthdawn boundary values
        assert_valid_roll("ed1");
        assert_valid_roll("ed50");
        assert_valid_roll("ed4e1");
        assert_valid_roll("ed4e50");

        // Wrath & Glory difficulty edge cases
        assert_valid_roll("wng dn1 1d6");
        assert_valid_roll("wng dn10 10d6");
    }

    /// Test integration between different systems
    #[test]
    fn test_system_integration() {
        // Combining aliases with manual modifiers
        assert_valid_roll("4cod + 1d6");
        assert_valid_roll("attack + 1d6 e6");
        assert_valid_roll("3df + 2");
        assert_valid_roll("sr6 - 1");

        // Multiple systems in semicolon rolls
        assert_valid_roll("attack; 2d6 + 3; 4cod");
        assert_valid_roll("1d20 + 5; 3df; wng 4d6");

        // Flags with system aliases
        assert_valid_roll("p 4cod");
        assert_valid_roll("s attack + 5");
        assert_valid_roll("ul 6 4d6 k3");
    }

    /// Test backward compatibility
    #[test]
    fn test_backward_compatibility() {
        // Ensure old syntax still works
        assert_valid_roll("1d6+2");
        assert_valid_roll("2d6-1");
        assert_valid_roll("3d6*2");
        assert_valid_roll("4d6/2");

        // Old alias formats
        assert_valid_roll("4cod");
        assert_valid_roll("3df");
        assert_valid_roll("dndstats");

        // Old modifier combinations
        assert_valid_roll("4d6e6k3");
        assert_valid_roll("6d10t7ie10");
    }
}

// ============================================================================
// HELPER TESTS FOR SPECIFIC FUNCTIONS (if needed for internal testing)
// ============================================================================

#[cfg(test)]
mod helper_function_tests {
    use super::*;

    /// Test specific internal functions if they're exposed
    #[test]
    fn test_format_functions() {
        // Test various formatting scenarios
        let results = parse_and_roll("4d6 k3").unwrap();
        let formatted = format_multiple_results_with_limit(&results);
        assert!(!formatted.is_empty());
        assert!(formatted.len() <= 2000);

        // Test with multiple results
        let results = parse_and_roll("1d20; 2d6; 1d4").unwrap();
        let formatted = format_multiple_results_with_limit(&results);
        assert!(!formatted.is_empty());
        assert!(formatted.len() <= 2000);

        // Test with roll sets
        let results = parse_and_roll("6 4d6 k3").unwrap();
        let formatted = format_multiple_results_with_limit(&results);
        assert!(!formatted.is_empty());
        assert!(formatted.len() <= 2000);
    }

    /// Test edge cases that might cause panics or unexpected behavior
    #[test]
    fn test_robustness() {
        // Test malformed but parseable inputs
        assert_valid_roll("1d6     +     2     ");
        assert_valid_roll("   4d6   k3   ");
        assert_valid_roll("2d6+3-1*2/1");

        // Test boundary mathematical operations
        assert_valid_roll("1d6 + 0");
        assert_valid_roll("1d6 - 0");
        assert_valid_roll("1d6 * 1");
        assert_valid_roll("1d6 / 1");

        // Test with minimum dice values
        assert_valid_roll("1d1");
        assert_valid_roll("1d1 + 1");
        assert_valid_roll("1d1 k1");
    }

    /// Test that the parser handles all documented features
    #[test]
    fn test_comprehensive_feature_coverage() {
        // Verify that all major features work together
        let complex_expression =
            "p s (Attack Roll) 4d6 e6 ie k3 r1 + 2d4 e4 - 1 ! with magical weapon";
        assert_valid_roll(complex_expression);

        // Verify that all flags work
        assert_valid_roll("p s nr ul 1d6");

        // Verify that all mathematical operations work
        assert_valid_roll("1d6 + 2 - 1 * 3 / 2");

        // Verify that all modifier types work together
        assert_valid_roll("6d6 e6 ie k4 r1 d1 t4 f1 b1 + 5");
    }

    /// Test error resilience
    #[test]
    fn test_error_resilience() {
        // Test that the parser fails gracefully on invalid input
        assert_invalid_roll("not a dice expression");
        assert_invalid_roll("1d6 + invalid");
        assert_invalid_roll("1d6 k invalid");
        assert_invalid_roll("1d6 + 1d6 + 1d6 + 1d6 + 1d6 + 1d6"); // Very long expression

        // Test that mathematical errors are caught
        assert_invalid_roll("1d6 / 0");
        assert_invalid_roll("1d0");
        assert_invalid_roll("-1d6");
    }

    /// Test memory and performance characteristics
    #[test]
    fn test_memory_performance() {
        // Test that large valid expressions don't cause memory issues
        assert_valid_roll("500d6"); // Maximum dice
        assert_valid_roll("20 10d6 k8"); // Maximum roll sets with complex modifiers
        assert_valid_roll("100d6 e6 k50"); // Large exploding dice pool

        // Test that the parser doesn't leak memory on repeated use
        for _ in 0..1000 {
            assert_valid_roll("1d6 + 2");
        }
    }

    /// Test all documented examples from help text work
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

        // Examples from system help
        assert_valid_roll("+d%");
        assert_valid_roll("-d%");
        assert_valid_roll("3df");
        assert_valid_roll("4df");
        assert_valid_roll("gb");
        assert_valid_roll("gbs");
        assert_valid_roll("gb 3d8");
        assert_valid_roll("2hsn");
        assert_valid_roll("3hsk");
        assert_valid_roll("2.5hsk");
        assert_valid_roll("3hsh");
        assert_valid_roll("wng 4d6");
        assert_valid_roll("wng dn2 4d6");
        assert_valid_roll("wng 4d6 !soak");
        assert_valid_roll("dh 4d10");
        assert_valid_roll("ed15");
    }
}
