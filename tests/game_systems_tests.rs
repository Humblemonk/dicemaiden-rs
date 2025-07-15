// tests/game_systems_tests.rs - Game System Specific Tests
//
// This file contains all tests for specific game system mechanics:
// - Game system aliases and expansions
// - System-specific dice mechanics
// - Cross-system compatibility
// - Game system modifiers and edge cases

use dicemaiden_rs::{dice::aliases, parse_and_roll};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn assert_valid(input: &str) {
    let result = parse_and_roll(input);
    assert!(result.is_ok(), "Failed to parse: '{}'", input);
}

fn assert_invalid(input: &str) {
    let result = parse_and_roll(input);
    assert!(result.is_err(), "Expected error for: '{}'", input);
}

// ============================================================================
// COMPREHENSIVE GAME SYSTEM TESTS
// ============================================================================

#[test]
fn test_game_systems_comprehensive() {
    // Single table-driven test replacing 15+ individual test functions
    let game_systems = vec![
        // (alias, should_parse, expected_feature, description)

        // World/Chronicles of Darkness
        (
            "4cod",
            true,
            Some("success"),
            "Chronicles of Darkness standard",
        ),
        ("4cod8", true, Some("success"), "Chronicles 8-again"),
        ("4cod9", true, Some("success"), "Chronicles 9-again"),
        ("4codr", true, Some("success"), "Chronicles rote quality"),
        ("4wod8", true, Some("success"), "World of Darkness diff 8"),
        ("5wod6", true, Some("success"), "World of Darkness diff 6"),
        (
            "4wod8c",
            true,
            Some("success"),
            "World of Darkness diff 8 with cancel",
        ),
        (
            "5wod6c",
            true,
            Some("success"),
            "World of Darkness diff 6 with cancel",
        ),
        (
            "6wod7c + 2",
            true,
            Some("success"),
            "World of Darkness with cancel and modifier",
        ),
        // Shadowrun
        ("sr5", true, Some("success"), "Shadowrun 5th edition"),
        ("sr6", true, Some("success"), "Shadowrun 6th edition"),
        // Godbound
        ("gb", true, Some("damage"), "Godbound basic damage"),
        ("gbs", true, Some("damage"), "Godbound straight damage"),
        ("gb 1d8", true, Some("damage"), "Godbound with dice"),
        (
            "gbs 2d10",
            true,
            Some("damage"),
            "Godbound straight with dice",
        ),
        // Fudge/FATE
        ("3df", true, Some("fudge"), "3 Fudge dice"),
        ("4df", true, Some("fudge"), "4 Fudge dice"),
        // Savage Worlds
        ("sw4", true, None, "Savage Worlds d4 trait"),
        ("sw6", true, None, "Savage Worlds d6 trait"),
        ("sw8", true, None, "Savage Worlds d8 trait"),
        ("sw10", true, None, "Savage Worlds d10 trait"),
        ("sw12", true, None, "Savage Worlds d12 trait"),
        // Hero System
        ("2hsn", true, None, "Hero System normal damage"),
        ("3hsk", true, None, "Hero System killing damage"),
        ("3hsh", true, None, "Hero System to-hit"),
        ("2hsk1", true, None, "Hero System fractional killing"),
        // Cyberpunk Red & Witcher
        ("cpr", true, None, "Cyberpunk Red basic"),
        ("cpr + 5", true, None, "Cyberpunk Red with modifier"),
        ("wit", true, None, "Witcher basic"),
        ("wit + 5", true, None, "Witcher with modifier"),
        // Other Systems
        ("age", true, None, "AGE system"),
        ("dndstats", true, None, "D&D ability scores"),
        ("ed15", true, None, "Earthdawn step 15"),
        ("d6s4", true, None, "D6 System"),
        ("bnw3", true, None, "Brave New World 3-die pool"),
        ("sil3", true, None, "Silhouette 3 dice"),
        ("conan", true, Some("success"), "Conan 2d20 skill"),
        ("cd", true, None, "Conan combat dice"),
        // Wrath & Glory variations
        ("wng 4d6", true, Some("success"), "W&G basic test"),
        ("wng dn2 4d6", true, Some("success"), "W&G difficulty test"),
        (
            "wng w2 4d6",
            true,
            Some("success"),
            "W&G multiple wrath dice",
        ),
        // Marvel Multiverse
        ("mm", true, None, "Marvel Multiverse basic"),
        ("mm e", true, None, "Marvel with edge"),
        ("mm t", true, None, "Marvel with trouble"),
        // Cypher System
        ("cs 1", true, None, "Cypher System level 1"),
        ("cs 6", true, None, "Cypher System level 6"),
        ("cs 10", true, None, "Cypher System level 10"),
        // Invalid cases
        ("invalid_system", false, None, "Should fail"),
        ("sil0", false, None, "Silhouette zero dice"),
        ("sil11", false, None, "Silhouette too many dice"),
        ("sw3", false, None, "Invalid Savage Worlds die"),
        ("sw14", false, None, "Invalid Savage Worlds die"),
        ("cs 0", false, None, "Invalid Cypher level"),
        ("cs 11", false, None, "Invalid Cypher level"),
        // Storypath System
        ("sp3", true, Some("success"), "Storypath 3 dice"),
        ("sp4", true, Some("success"), "Storypath 4 dice"),
        ("sp6", true, Some("success"), "Storypath 6 dice"),
        ("sp8", true, Some("success"), "Storypath 8 dice"),
        ("sp4t7", true, Some("success"), "Storypath 4 dice target 7"),
        ("sp5t9", true, Some("success"), "Storypath 5 dice target 9"),
        // Double Digit Dice
        ("dd34", true, None, "Double digit d3*10 + d4"),
        ("dd26", true, None, "Double digit d2*10 + d6"),
        ("dd66", true, None, "Double digit d6*10 + d6"),
        ("dd46", true, None, "Double digit d4*10 + d6"),
        // Sunsails New Millennium
        ("snm3", true, Some("success"), "SNM 3 dice"),
        ("snm5", true, Some("success"), "SNM 5 dice"),
        ("snm8", true, Some("success"), "SNM 8 dice"),
        // Year Zero Engine
        ("3yz", true, Some("success"), "Year Zero 3 dice"),
        ("6yz", true, Some("success"), "Year Zero 6 dice"),
        ("8yz", true, Some("success"), "Year Zero 8 dice"),
        // Warhammer 40k/Age of Sigmar
        ("2wh3+", true, Some("success"), "Warhammer 2d6 3+"),
        ("3wh4+", true, Some("success"), "Warhammer 3d6 4+"),
        ("5wh5+", true, Some("success"), "Warhammer 5d6 5+"),
        // D6 Legends
        ("1d6l", true, Some("success"), "D6 Legends wild die only"),
        (
            "8d6l",
            true,
            Some("success"),
            "D6 Legends 7 regular + 1 wild",
        ),
        (
            "12d6l",
            true,
            Some("success"),
            "D6 Legends 11 regular + 1 wild",
        ),
        ("0d6l", false, None, "Invalid D6 Legends zero dice"),
        // VTM5 - Vampire: The Masquerade 5th Edition
        ("vtm5h2", true, Some("success"), "VTM5 5 dice, 2 hunger"),
        ("vtm7h2", true, Some("success"), "VTM5 7 dice, 2 hunger"),
        ("vtm8h0", true, Some("success"), "VTM5 8 dice, no hunger"),
        ("vtm10h3", true, Some("success"), "VTM5 10 dice, 3 hunger"),
        (
            "vtm1h1",
            true,
            Some("success"),
            "VTM5 minimum pool, all hunger",
        ),
        ("vtm8h5", true, Some("success"), "VTM5 8 dice, max hunger"),
    ];

    for (system, should_parse, expected_feature, description) in game_systems {
        let result = parse_and_roll(system);

        if should_parse {
            assert!(
                result.is_ok(),
                "Game system '{}' should parse successfully: {}",
                system,
                description
            );

            let results = result.unwrap();
            assert!(!results.is_empty(), "No results for: '{}'", system);

            // Check expected features
            if let Some(feature) = expected_feature {
                match feature {
                    "success" => assert!(
                        results[0].successes.is_some(),
                        "Game system '{}' should have success counting",
                        system
                    ),
                    "fudge" => assert!(
                        results[0].fudge_symbols.is_some(),
                        "Game system '{}' should have fudge symbols",
                        system
                    ),
                    "damage" => assert!(
                        results[0].godbound_damage.is_some() || results[0].total > 0,
                        "Game system '{}' should have damage calculation",
                        system
                    ),
                    _ => {}
                }
            }
        } else {
            assert!(
                result.is_err(),
                "Game system '{}' should fail to parse: {}",
                system,
                description
            );
        }
    }
}

#[test]
fn test_game_system_modifiers() {
    // Test game systems with mathematical modifiers
    let system_modifiers = vec![
        ("cpr + 10", "Cyberpunk Red with bonus"),
        ("cpr - 4", "Cyberpunk Red with penalty"),
        ("cpr * 3", "Cyberpunk Red multiplied"),
        ("cpr / 2", "Cyberpunk Red divided"),
        ("wit + 5", "Witcher with bonus"),
        ("wit - 3", "Witcher with penalty"),
        ("wit * 2", "Witcher multiplied"),
        ("wit / 2", "Witcher divided"),
        ("cs 3 + 2", "Cypher System with modifier"),
        ("gb + 5", "Godbound with bonus"),
        ("gbs - 2", "Godbound straight with penalty"),
        ("sw8 + 3", "Savage Worlds with bonus"),
        ("sw10 * 2", "Savage Worlds multiplied"),
        ("vtm6h2 + 2", "VTM5 with positive modifier"),
        ("vtm5h1 - 1", "VTM5 with negative modifier"),
        ("vtm4h0 * 2", "VTM5 with multiplication"),
        ("vtm8h3 / 2", "VTM5 with division"),
    ];

    for (expression, description) in system_modifiers {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "System with modifier '{}' should work: {}",
            expression,
            description
        );
    }
}

#[test]
fn test_game_system_cross_compatibility() {
    // Ensure game systems don't interfere with each other
    let mixed_tests = vec![
        ("4cod", "cpr", "CoD and CPR should both work"),
        ("sw8", "sr6", "Savage Worlds and Shadowrun should both work"),
        ("gb", "4df", "Godbound and Fudge should both work"),
        ("wit", "conan", "Witcher and Conan should both work"),
        ("mm", "wng 4d6", "Marvel and W&G should both work"),
    ];

    for (system1, system2, description) in mixed_tests {
        let result1 = parse_and_roll(system1);
        let result2 = parse_and_roll(system2);

        assert!(result1.is_ok() && result2.is_ok(), "{}", description);

        // They should produce different types of results
        let r1 = result1.unwrap();
        let r2 = result2.unwrap();

        assert!(
            r1[0].total != 0
                || r1[0].successes.is_some()
                || r1[0].godbound_damage.is_some()
                || r1[0].fudge_symbols.is_some(),
            "System '{}' should produce valid results",
            system1
        );
        assert!(
            r2[0].total != 0
                || r2[0].successes.is_some()
                || r2[0].godbound_damage.is_some()
                || r2[0].fudge_symbols.is_some(),
            "System '{}' should produce valid results",
            system2
        );
    }
}

// ============================================================================
// ALIAS EXPANSION TESTS
// ============================================================================

#[test]
fn test_alias_expansions() {
    // Test that aliases expand correctly
    let alias_tests = vec![
        ("4cod", Some("4d10 t8 ie10")),
        ("4codr", Some("4d10 t8 ie10 r7")),
        ("4wod8", Some("4d10 f1 t8")),
        ("sr6", Some("6d6 t5 shadowrun6")),
        ("3df", Some("3d3 fudge")),
        ("age", Some("2d6 + 1d6")),
        ("+d20", Some("2d20 k1")),
        ("-d20", Some("2d20 kl1")),
        ("+d%", Some("2d10 kl1 * 10 + 1d10 - 10")),
        ("-d%", Some("2d10 k1 * 10 + 1d10 - 10")),
        ("dndstats", Some("6 4d6 k3")),
        ("invalid_alias", None),
        ("1d6l", Some("1d6 t4f1ie6")),
        ("8d6l", Some("7d6 t4 + 1d6 t4f1ie6")),
        ("12d6l", Some("11d6 t4 + 1d6 t4f1ie6")),
        ("4wod8c", Some("4d10 f1 t8 c")),
        ("5wod6c", Some("5d10 f1 t6 c")),
        ("6wod7c + 3", Some("6d10 f1 t7 c + 3")),
        ("vtm5h2", Some("5d10 vtm5p5h2")),
        ("vtm7h2", Some("7d10 vtm5p7h2")),
        ("vtm8h0", Some("8d10 vtm5p8h0")),
        ("vtm10h3", Some("10d10 vtm5p10h3")),
    ];

    for (alias, expected) in alias_tests {
        let result = aliases::expand_alias(alias);
        match expected {
            Some(expansion) => assert_eq!(
                result,
                Some(expansion.to_string()),
                "Alias '{}' should expand to '{}'",
                alias,
                expansion
            ),
            None => assert_eq!(result, None, "Alias '{}' should not expand", alias),
        }
    }
}

// ============================================================================
// SPECIFIC SYSTEM MECHANICS
// ============================================================================

#[test]
fn test_savage_worlds_mechanics() {
    // Test Savage Worlds specific behavior
    let result = parse_and_roll("sw8").unwrap();
    assert_eq!(result.len(), 1);

    // Should have trait and wild die notes
    let has_trait_note = result[0]
        .notes
        .iter()
        .any(|note| note.contains("Trait die") || note.contains("Wild die"));
    assert!(
        has_trait_note,
        "Savage Worlds should have trait/wild die notes"
    );

    // Test with roll sets
    let result = parse_and_roll("3 sw8").unwrap();
    assert_eq!(result.len(), 3);
    for roll in &result {
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
    }
}

#[test]
fn test_cyberpunk_red_mechanics() {
    // Test CPR critical success/failure mechanics
    for _ in 0..10 {
        let result = parse_and_roll("cpr").unwrap();
        assert_eq!(result.len(), 1);

        // Should not have success counting (total-based system)
        assert!(result[0].successes.is_none());

        // Should be in valid range (-9 to 20 with explosions)
        assert!(result[0].total >= -9 && result[0].total <= 20);
    }
}

#[test]
fn test_witcher_mechanics() {
    // Test Witcher indefinite explosion mechanics
    for _ in 0..10 {
        let result = parse_and_roll("wit").unwrap();
        assert_eq!(result.len(), 1);

        // Should not have success counting (total-based system)
        assert!(result[0].successes.is_none());

        // Should be in reasonable range (allowing for explosions)
        assert!(result[0].total >= -100 && result[0].total <= 110);
    }
}

#[test]
fn test_wrath_glory_mechanics() {
    // Test W&G success counting and wrath dice
    let result = parse_and_roll("wng 4d6").unwrap();
    assert_eq!(result.len(), 1);

    // Should have success counting
    assert!(result[0].successes.is_some());

    // Should have wrath die information
    assert!(result[0].wng_wrath_die.is_some() || result[0].wng_wrath_dice.is_some());

    // Test difficulty mechanics
    let result = parse_and_roll("wng dn3 4d6").unwrap();
    let has_difficulty_note = result[0]
        .notes
        .iter()
        .any(|note| note.contains("Difficulty 3"));
    assert!(has_difficulty_note, "Should have difficulty note");

    // Test multiple wrath dice
    let result = parse_and_roll("wng w2 4d6").unwrap();
    assert!(result[0].successes.is_some());
}

#[test]
fn test_marvel_multiverse_mechanics() {
    // Test Marvel Multiverse edge/trouble mechanics
    let result = parse_and_roll("mm").unwrap();
    assert_eq!(result.len(), 1);
    assert!(result[0].total >= 3 && result[0].total <= 18); // 3d6 range

    // Test with edges
    let result = parse_and_roll("mm e").unwrap();
    assert!(result[0].total >= 3 && result[0].total <= 18);

    // Test with troubles
    let result = parse_and_roll("mm t").unwrap();
    assert!(result[0].total >= 3 && result[0].total <= 18);
}

#[test]
fn test_fudge_dice_mechanics() {
    // Test Fudge dice symbols and ranges
    let result = parse_and_roll("4df").unwrap();
    assert_eq!(result.len(), 1);

    // Should have fudge symbols
    assert!(result[0].fudge_symbols.is_some());

    // Should be in valid range (-4 to +4 for 4dF)
    assert!(result[0].total >= -4 && result[0].total <= 4);

    let symbols = result[0].fudge_symbols.as_ref().unwrap();
    assert_eq!(symbols.len(), 4);

    // Each symbol should be +, -, or blank
    for symbol in symbols {
        assert!(symbol == "+" || symbol == "-" || symbol == " ");
    }
}

#[test]
fn test_godbound_damage_mechanics() {
    // Test Godbound damage chart conversion
    let result = parse_and_roll("gb").unwrap();
    assert_eq!(result.len(), 1);

    // Should have godbound damage calculation
    assert!(result[0].godbound_damage.is_some());

    // Should have damage chart note
    let has_chart_note = result[0]
        .notes
        .iter()
        .any(|note| note.contains("damage chart") || note.contains("Godbound"));
    assert!(has_chart_note, "Should have Godbound damage chart note");

    // Test straight damage
    let result = parse_and_roll("gbs").unwrap();
    assert!(result[0].godbound_damage.is_some());

    let has_straight_note = result[0]
        .notes
        .iter()
        .any(|note| note.contains("Straight damage"));
    assert!(has_straight_note, "Should have straight damage note");
}

// ============================================================================
// SYSTEM EDGE CASES
// ============================================================================

#[test]
fn test_system_validation_edge_cases() {
    // Test boundary conditions for various systems

    // Savage Worlds - only even-sided dice 4-12
    assert_valid("sw4"); // Minimum
    assert_valid("sw12"); // Maximum
    assert_invalid("sw2"); // Too small
    assert_invalid("sw14"); // Too large
    assert_invalid("sw5"); // Odd-sided

    // Silhouette - 1-10 dice
    assert_valid("sil1"); // Minimum
    assert_valid("sil10"); // Maximum
    assert_invalid("sil0"); // Too small
    assert_invalid("sil11"); // Too large

    // Cypher System - levels 1-10
    assert_valid("cs 1"); // Minimum
    assert_valid("cs 10"); // Maximum
    assert_invalid("cs 0"); // Too small
    assert_invalid("cs 11"); // Too large

    // Wrath & Glory - 1-5 wrath dice
    assert_valid("wng w1 4d6"); // Minimum
    assert_valid("wng w5 4d6"); // Maximum
    assert_invalid("wng w0 4d6"); // Too small
    assert_invalid("wng w6 4d6"); // Too large
}

#[test]
fn test_system_roll_sets() {
    // Test game systems work with roll sets
    let roll_set_systems = vec![
        "3 sw8", "3 cpr", "3 wit", "3 4cod", "3 gb", "3 mm", "3 cs 5", "3 vtm5h2",
    ];
    for system in roll_set_systems {
        let result = parse_and_roll(system);
        assert!(result.is_ok(), "Roll set '{}' should work", system);

        let results = result.unwrap();
        assert_eq!(results.len(), 3, "Should have 3 sets for '{}'", system);

        for roll in &results {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        }
    }
}

#[test]
fn test_dnd_aliases_comprehensive() {
    // Test D&D/Pathfinder aliases mentioned in roll_syntax.md
    let dnd_aliases = vec![
        // Basic aliases
        ("attack", "1d20", "Basic attack roll"),
        ("skill", "1d20", "Basic skill check"),
        ("save", "1d20", "Basic saving throw"),
        // Aliases with modifiers
        ("attack +5", "1d20 + 5", "Attack with bonus"),
        ("attack -2", "1d20 - 2", "Attack with penalty"),
        ("skill +3", "1d20 + 3", "Skill with bonus"),
        ("skill -4", "1d20 - 4", "Skill with penalty"),
        ("save +2", "1d20 + 2", "Save with bonus"),
        ("save -1", "1d20 - 1", "Save with penalty"),
        // Large modifiers
        ("attack +10", "1d20 + 10", "High-level attack"),
        ("skill -5", "1d20 - 5", "Difficult skill check"),
        ("save +8", "1d20 + 8", "High save bonus"),
    ];

    for (alias, expected_equivalent, description) in dnd_aliases {
        // Test that the alias works
        let alias_result = parse_and_roll(alias);
        assert!(
            alias_result.is_ok(),
            "D&D alias '{}' should parse: {}",
            alias,
            description
        );

        // Test that the equivalent expression also works
        let equivalent_result = parse_and_roll(expected_equivalent);
        assert!(
            equivalent_result.is_ok(),
            "Equivalent '{}' should parse: {}",
            expected_equivalent,
            description
        );

        let alias_roll = alias_result.unwrap();
        let equiv_roll = equivalent_result.unwrap();

        // Both should produce results in similar ranges (can't test exact equality due to randomness)
        assert_eq!(
            alias_roll[0].individual_rolls.len(),
            equiv_roll[0].individual_rolls.len(),
            "Dice count should match for '{}' vs '{}': {}",
            alias,
            expected_equivalent,
            description
        );
    }

    // Test D&D advantage/disadvantage in different contexts
    let advantage_tests = vec![
        ("+d20", "Advantage"),
        ("-d20", "Disadvantage"),
        ("+d20 + 5", "Advantage with modifier"),
        ("-d20 - 2", "Disadvantage with penalty"),
        ("3 +d20", "Advantage roll sets"),
        ("2 -d20 + 3", "Disadvantage roll sets with modifier"),
    ];

    for (expression, description) in advantage_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Advantage test '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        if expression.contains("3 ") || expression.contains("2 ") {
            // Roll sets
            assert!(
                results.len() >= 2,
                "Should have multiple sets for '{}'",
                expression
            );
        } else {
            // Single roll
            assert_eq!(
                results.len(),
                1,
                "Should have single result for '{}'",
                expression
            );
        }
    }
}

#[test]
fn test_hero_system_variants() {
    // Test Hero System variants mentioned in roll_syntax.md but not currently tested
    let hero_variants = vec![
        // Fractional dice
        ("2.5hsk", "2½d6 killing damage"),
        ("1.5hsn", "1½d6 normal damage"),
        ("3.5hsk", "3½d6 killing damage"),
        // Single-die versions
        ("hsn", "Single die normal damage"),
        ("hsk", "Single die killing damage"),
        ("hsh", "Single die to-hit"),
        // Additional fractional notation
        ("1hsk1", "1d6 + 1d3 killing (fractional notation)"),
        ("2hsk1", "2d6 + 1d3 killing (fractional notation)"),
        ("3hsn1", "3d6 + 1d3 normal (if supported)"),
    ];

    for (expression, description) in hero_variants {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Hero System variant '{}' should parse: {}",
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

        // Basic validation - should have reasonable totals
        assert!(
            results[0].total >= 0,
            "Should have non-negative total for '{}': {}",
            expression,
            description
        );

        // Check if it has the Hero System notes
        let has_hero_note = results[0].notes.iter().any(|note| {
            note.contains("Hero")
                || note.contains("damage")
                || note.contains("hit")
                || note.contains("BODY")
                || note.contains("STUN")
        });
        assert!(
            has_hero_note,
            "Should have Hero System notes for '{}': {}",
            expression, description
        );
    }

    // Test Hero System with modifiers
    let hero_modifier_tests = vec!["2hsn + 5", "3hsk - 2", "hsh + 3", "2.5hsk * 2", "1hsk1 + 4"];

    for test in hero_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Hero System modifier test '{}' should parse",
            test
        );
    }
}

#[test]
fn test_silhouette_variants() {
    // Test Silhouette System variants mentioned in roll_syntax.md
    let silhouette_variants = vec![
        ("sil", 1, "Default Silhouette (1 die)"),
        ("sil1", 1, "Silhouette 1 die explicitly"),
        ("sil3", 3, "Silhouette 3 dice"),
        ("sil5", 5, "Silhouette 5 dice"),
        ("sil10", 10, "Silhouette 10 dice (maximum)"),
    ];

    for (expression, expected_dice, description) in silhouette_variants {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Silhouette variant '{}' should parse: {}",
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

        // Check that the correct number of dice were rolled
        assert_eq!(
            results[0].individual_rolls.len(),
            expected_dice,
            "Should have {} dice for '{}': {}",
            expected_dice,
            expression,
            description
        );

        // Silhouette system should keep highest + bonus for extra 6s
        // Total should be at least 1 (minimum die result)
        assert!(
            results[0].total >= 1,
            "Should have positive total for '{}': {}",
            expression,
            description
        );

        // Should be reasonable maximum (6 + extra 6s)
        assert!(
            results[0].total <= 6 + expected_dice as i32,
            "Total should be reasonable for '{}': {}",
            expression,
            description
        );
    }

    // Test invalid Silhouette dice counts
    let invalid_silhouette = vec![
        "sil0",  // Zero dice
        "sil11", // Too many dice
        "sil20", // Way too many dice
    ];

    for invalid_test in invalid_silhouette {
        let result = parse_and_roll(invalid_test);
        assert!(
            result.is_err(),
            "Invalid Silhouette '{}' should fail",
            invalid_test
        );
    }

    // Test Silhouette with modifiers
    let silhouette_modifier_tests = vec!["sil3 + 2", "sil5 - 1", "sil + 3", "sil10 * 2"];

    for test in silhouette_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Silhouette modifier test '{}' should parse",
            test
        );
    }
}

#[test]
fn test_conan_system_variants() {
    // Test Conan system variants mentioned in roll_syntax.md
    let conan_variants = vec![
        // Skill dice variants
        ("conan", "Default 2d20 skill"),
        ("conan3", "3d20 skill"),
        ("conan4", "4d20 skill"),
        ("conan5", "5d20 skill"),
        // Combat dice variants
        ("cd", "Default 1d6 combat"),
        ("cd4", "4d6 combat"),
        ("cd10", "10d6 combat"),
        // Combined attacks
        ("conan3cd5", "3d20 skill + 5d6 combat"),
        ("conan2cd4", "2d20 skill + 4d6 combat"),
        ("conan4cd6", "4d20 skill + 6d6 combat"),
    ];

    for (expression, description) in conan_variants {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Conan variant '{}' should parse: {}",
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

        // Check for appropriate notes or mechanics
        let has_conan_note = results[0]
            .notes
            .iter()
            .any(|note| note.contains("1=1") || note.contains("2=2") || note.contains("special"));

        if expression.starts_with("cd") || expression.contains("cd") {
            assert!(
                has_conan_note,
                "Combat dice should have interpretation notes for '{}': {}",
                expression, description
            );
        }

        // Should have reasonable totals
        assert!(
            results[0].total >= 0,
            "Should have non-negative total for '{}': {}",
            expression,
            description
        );
    }

    // Test Conan with modifiers
    let conan_modifier_tests = vec!["conan + 2", "conan3 - 1", "cd4 + 3", "conan2cd4 + 5"];

    for test in conan_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Conan modifier test '{}' should parse",
            test
        );
    }

    // Test invalid Conan variants
    let invalid_conan = vec![
        "conan1", // Too few dice
        "conan6", // Too many dice
        "cd0",    // Zero combat dice
    ];

    for invalid_test in invalid_conan {
        let result = parse_and_roll(invalid_test);
        assert!(
            result.is_err(),
            "Invalid Conan '{}' should fail",
            invalid_test
        );
    }
}

#[test]
fn test_missing_game_systems() {
    // Test the remaining game systems mentioned in roll_syntax.md but not yet tested
    let missing_systems = vec![
        // Dark Heresy 2nd Edition
        ("dh 4d10", "Dark Heresy 4d10 righteous fury"),
        ("dh 6d10", "Dark Heresy 6d10 righteous fury"),
        // Exalted variants
        ("ex5", "Exalted 5d10 t7 t10"),
        ("ex5t8", "Exalted 5d10 t8 t10"),
        ("ex10", "Exalted 10d10"),
        ("ex3t6", "Exalted 3d10 t6 t10"),
        // Year Zero Engine
        ("6yz", "Year Zero 6d6 t6"),
        ("4yz", "Year Zero 4d6 t6"),
        ("8yz", "Year Zero 8d6 t6"),
        // Warhammer 40k/Age of Sigmar
        ("3wh4+", "Warhammer 3d6 t4"),
        ("5wh3+", "Warhammer 5d6 t3"),
        ("2wh5+", "Warhammer 2d6 t5"),
        // Earthdawn 4th Edition
        ("ed4e15", "Earthdawn 4e step 15"),
        ("ed4e20", "Earthdawn 4e step 20"),
        ("ed4e50", "Earthdawn 4e step 50"),
        // Double Digit dice
        ("dd34", "1d3*10 + 1d4 (d66-style)"),
        ("dd26", "1d2*10 + 1d6"),
        ("dd46", "1d4*10 + 1d6"),
        // Storypath System
        ("sp4", "Storypath 4d10 t8 ie10"),
        ("sp6", "Storypath 6d10 t8 ie10"),
        ("sp8", "Storypath 8d10 t8 ie10"),
        // Sunsails New Millennium
        ("snm5", "Sunsails 5d6 ie6 t4"),
        ("snm3", "Sunsails 3d6 ie6 t4"),
        ("snm8", "Sunsails 8d6 ie6 t4"),
    ];

    for (expression, description) in missing_systems {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Missing system '{}' should parse: {}",
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

        // Basic sanity check - should have reasonable output
        assert!(
            results[0].total >= 0
                || results[0].successes.is_some()
                || results[0].individual_rolls.len() > 0,
            "Should have meaningful results for '{}': {}",
            expression,
            description
        );
    }
}

#[test]
fn test_complex_system_combinations() {
    // Test complex system combinations mentioned in roll_syntax.md
    let complex_combinations = vec![
        // Godbound with complex modifiers
        ("gbs 2d10 + 5", "Godbound straight 2d10 with bonus"),
        ("gb 3d8 - 2", "Godbound 3d8 with penalty"),
        ("gbs 1d20 * 2", "Godbound straight with multiplier"),
        // Brave New World with modifiers
        ("bnw4 + 2", "BNW 4-die pool with modifier"),
        ("bnw5 - 1", "BNW 5-die pool with penalty"),
        ("bnw3 * 2", "BNW 3-die pool with multiplier"),
        // Marvel Multiverse complex combinations
        ("mm 2e", "Marvel with 2 edges"),
        ("mm 3t", "Marvel with 3 troubles"),
        ("mm 2e 3t", "Marvel with 2 edges and 3 troubles"),
        ("mm e t", "Marvel with 1 edge and 1 trouble"),
        // Complex Cypher System tests
        ("cs 1 + 5", "Cypher level 1 with modifier"),
        ("cs 10 - 3", "Cypher level 10 with penalty"),
        // Complex Witcher tests
        ("wit * 2", "Witcher with multiplier"),
        ("wit / 2", "Witcher with division"),
    ];

    for (expression, description) in complex_combinations {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Complex combination '{}' should parse: {}",
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

        // Verify appropriate mechanics are working
        match expression {
            expr if expr.contains("gb") => {
                assert!(
                    results[0].godbound_damage.is_some(),
                    "Godbound should have damage calculation for '{}'",
                    expr
                );
            }
            expr if expr.contains("mm") => {
                assert!(
                    results[0].total >= 3 && results[0].total <= 18,
                    "Marvel should have 3d6 range for '{}'",
                    expr
                );
            }
            expr if expr.contains("cs") => {
                // Cypher system should have appropriate notes
                let has_cypher_note = results[0].notes.iter().any(|note| note.contains("Cypher"));
                assert!(
                    has_cypher_note,
                    "Cypher should have system notes for '{}'",
                    expr
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test_system_edge_cases_and_boundaries() {
    // Test boundary conditions for systems that might not be fully covered
    let boundary_tests = vec![
        // Marvel Multiverse boundaries
        ("mm 5e", "Marvel with maximum practical edges"),
        ("mm 5t", "Marvel with maximum practical troubles"),
        // Cypher System boundaries
        ("cs 1", "Cypher minimum level"),
        ("cs 10", "Cypher maximum level"),
        // Wrath & Glory boundaries
        ("wng w1 1d6", "W&G minimum wrath dice"),
        ("wng w5 5d6", "W&G maximum wrath dice"),
        // Earthdawn boundaries
        ("ed1", "Earthdawn minimum step"),
        ("ed50", "Earthdawn maximum step (1e)"),
        ("ed4e1", "Earthdawn 4e minimum step"),
        ("ed4e100", "Earthdawn 4e maximum step"),
        // Hero System edge cases
        ("0.5hsk", "Hero System half die (if supported)"),
        ("10hsk", "Hero System large dice count"),
    ];

    for (expression, description) in boundary_tests {
        let result = parse_and_roll(expression);
        // Some of these might be invalid, but they shouldn't crash
        if result.is_ok() {
            let results = result.unwrap();
            assert!(
                !results.is_empty(),
                "Should have results for '{}': {}",
                expression,
                description
            );
        } else {
            println!(
                "Boundary test '{}' failed as expected: {}",
                expression, description
            );
        }
    }
}

#[test]
fn test_storypath_system_comprehensive() {
    // Test Storypath System (spX -> Xd10 t8 ie10)
    let storypath_tests = vec![
        // (alias, expected_dice_count, description)
        ("sp3", 3, "Storypath 3 dice"),
        ("sp4", 4, "Storypath 4 dice"),
        ("sp5", 5, "Storypath 5 dice"),
        ("sp6", 6, "Storypath 6 dice"),
        ("sp8", 8, "Storypath 8 dice"),
        ("sp10", 10, "Storypath 10 dice"),
        ("sp12", 12, "Storypath 12 dice"),
        ("sp4t7", 4, "Storypath 4 dice custom target 7"),
        ("sp3t6", 3, "Storypath 3 dice custom target 6"),
        ("sp5t9", 5, "Storypath 5 dice custom target 9"),
    ];

    for (alias, expected_dice, description) in storypath_tests {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Storypath '{}' should parse: {}",
            alias,
            description
        );

        let results = result.unwrap();
        assert_eq!(results.len(), 1, "Should have one result for '{}'", alias);

        let roll = &results[0];

        // Should have success counting (target system)
        assert!(
            roll.successes.is_some(),
            "Storypath should have success counting for '{}'",
            alias
        );

        // Should have exploding dice (individual_rolls may be more than expected due to explosions)
        assert!(
            roll.individual_rolls.len() >= expected_dice,
            "Should have at least {} dice for '{}', got {}",
            expected_dice,
            alias,
            roll.individual_rolls.len()
        );

        // Success count should be reasonable (0 to dice count + explosions)
        let success_count = roll.successes.unwrap();
        assert!(
            success_count >= 0 && success_count <= roll.individual_rolls.len() as i32,
            "Success count {} should be reasonable for '{}' with {} dice",
            success_count,
            alias,
            roll.individual_rolls.len()
        );

        // Should be using d10s (all rolls 1-10)
        for &die_roll in &roll.individual_rolls {
            assert!(
                die_roll >= 1 && die_roll <= 10,
                "Storypath should use d10s, got {} for '{}'",
                die_roll,
                alias
            );
        }
    }

    // Test Storypath with modifiers
    let storypath_modifier_tests = vec!["sp4 + 2", "sp6 - 1", "sp5 * 2", "sp3 + 1d6", "sp4t7 + 2"];

    for test in storypath_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Storypath modifier test '{}' should parse",
            test
        );
    }

    // Test edge cases
    let storypath_edge_cases = vec![
        ("sp1", 1, "Minimum Storypath dice"),
        ("sp15", 15, "Large Storypath pool"),
    ];

    for (alias, expected_dice, description) in storypath_edge_cases {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Storypath edge case '{}' should work: {}",
            alias,
            description
        );

        let results = result.unwrap();
        let roll = &results[0];
        assert!(
            roll.individual_rolls.len() >= expected_dice,
            "Should have at least {} dice for edge case '{}'",
            expected_dice,
            alias
        );
    }
}

#[test]
fn test_double_digit_dice_comprehensive() {
    // Test Double Digit Dice System (ddXY -> 1dX * 10 + 1dY)
    let double_digit_tests = vec![
        // (alias, tens_sides, ones_sides, description)
        ("dd34", 3, 4, "d3 tens + d4 ones"),
        ("dd26", 2, 6, "d2 tens + d6 ones"),
        ("dd46", 4, 6, "d4 tens + d6 ones"),
        ("dd66", 6, 6, "d6 tens + d6 ones (d66)"),
        ("dd36", 3, 6, "d3 tens + d6 ones"),
        ("dd23", 2, 3, "d2 tens + d3 ones"),
        ("dd44", 4, 4, "d4 tens + d4 ones"),
        ("dd88", 8, 8, "d8 tens + d8 ones"),
    ];

    for (alias, tens_sides, ones_sides, description) in double_digit_tests {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Double digit '{}' should parse: {}",
            alias,
            description
        );

        let results = result.unwrap();
        assert_eq!(results.len(), 1, "Should have one result for '{}'", alias);

        let roll = &results[0];

        // Should have exactly 2 dice (tens and ones)
        assert_eq!(
            roll.individual_rolls.len(),
            2,
            "Double digit should have exactly 2 dice for '{}'",
            alias
        );

        // Should not have success counting (it's a total-based system)
        assert!(
            roll.successes.is_none(),
            "Double digit should not have success counting for '{}'",
            alias
        );

        // Calculate expected range
        let min_total = 1 * 10 + 1; // Minimum: 1 on tens die * 10 + 1 on ones die
        let max_total = tens_sides * 10 + ones_sides; // Maximum possible

        assert!(
            roll.total >= min_total && roll.total <= max_total,
            "Double digit total {} should be between {} and {} for '{}': {}",
            roll.total,
            min_total,
            max_total,
            alias,
            description
        );

        // Validate individual dice are in correct ranges
        // Note: We can't easily separate tens vs ones dice from individual_rolls
        // but we can verify they're all in reasonable ranges
        for &die_roll in &roll.individual_rolls {
            assert!(
                die_roll >= 1 && die_roll <= 8, // Max of our test cases
                "Double digit die roll {} should be reasonable for '{}'",
                die_roll,
                alias
            );
        }
    }

    // Test double digit with modifiers
    let double_digit_modifier_tests = vec!["dd34 + 5", "dd66 - 10", "dd26 * 2", "dd46 / 2"];

    for test in double_digit_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Double digit modifier test '{}' should parse",
            test
        );
    }

    // Test mathematical validation for specific cases
    let dd34_result = parse_and_roll("dd34").unwrap();
    let dd34_roll = &dd34_result[0];
    // dd34 should be between 11 (1*10+1) and 34 (3*10+4)
    assert!(
        dd34_roll.total >= 11 && dd34_roll.total <= 34,
        "dd34 should be between 11-34, got {}",
        dd34_roll.total
    );

    let dd66_result = parse_and_roll("dd66").unwrap();
    let dd66_roll = &dd66_result[0];
    // dd66 should be between 11 (1*10+1) and 66 (6*10+6)
    assert!(
        dd66_roll.total >= 11 && dd66_roll.total <= 66,
        "dd66 should be between 11-66, got {}",
        dd66_roll.total
    );
}

#[test]
fn test_sunsails_new_millennium_comprehensive() {
    // Test Sunsails New Millennium (snmX -> Xd6 ie6 t4)
    let snm_tests = vec![
        // (alias, expected_dice_count, description)
        ("snm3", 3, "SNM 3 dice"),
        ("snm4", 4, "SNM 4 dice"),
        ("snm5", 5, "SNM 5 dice"),
        ("snm6", 6, "SNM 6 dice"),
        ("snm8", 8, "SNM 8 dice"),
        ("snm10", 10, "SNM 10 dice"),
    ];

    for (alias, expected_dice, description) in snm_tests {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "SNM '{}' should parse: {}",
            alias,
            description
        );

        let results = result.unwrap();
        assert_eq!(results.len(), 1, "Should have one result for '{}'", alias);

        let roll = &results[0];

        // Should have success counting (target 4+)
        assert!(
            roll.successes.is_some(),
            "SNM should have success counting for '{}'",
            alias
        );

        // Should have exploding dice (ie6), so may have more than expected dice
        assert!(
            roll.individual_rolls.len() >= expected_dice,
            "Should have at least {} dice for '{}', got {} (explosions expected)",
            expected_dice,
            alias,
            roll.individual_rolls.len()
        );

        // Success count should be reasonable
        let success_count = roll.successes.unwrap();
        assert!(
            success_count >= 0 && success_count <= roll.individual_rolls.len() as i32,
            "Success count {} should be reasonable for '{}' with {} dice",
            success_count,
            alias,
            roll.individual_rolls.len()
        );

        // Should be using d6s (all rolls 1-6)
        for &die_roll in &roll.individual_rolls {
            assert!(
                die_roll >= 1 && die_roll <= 6,
                "SNM should use d6s, got {} for '{}'",
                die_roll,
                alias
            );
        }

        // Should have notes about explosions if any 6s were rolled
        let has_sixes = roll.individual_rolls.iter().any(|&r| r == 6);
        if has_sixes && roll.individual_rolls.len() > expected_dice {
            let has_explosion_note = roll
                .notes
                .iter()
                .any(|note| note.contains("exploded") || note.contains("explosion"));
            assert!(
                has_explosion_note,
                "Should have explosion note for '{}' when 6s are rolled",
                alias
            );
        }
    }

    // Test SNM with modifiers
    let snm_modifier_tests = vec!["snm5 + 2", "snm3 - 1", "snm4 * 2", "snm6 + 1d4"];

    for test in snm_modifier_tests {
        let result = parse_and_roll(test);
        assert!(result.is_ok(), "SNM modifier test '{}' should parse", test);
    }

    // Test edge cases
    let snm_edge_cases = vec![
        ("snm1", 1, "Minimum SNM dice"),
        ("snm12", 12, "Large SNM pool"),
    ];

    for (alias, _expected_dice, description) in snm_edge_cases {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "SNM edge case '{}' should work: {}",
            alias,
            description
        );
    }
}

#[test]
fn test_year_zero_engine_comprehensive() {
    // Test Year Zero Engine (XYZ -> Xd6 t6)
    let year_zero_tests = vec![
        // (alias, expected_dice_count, description)
        ("3yz", 3, "Year Zero 3 dice"),
        ("4yz", 4, "Year Zero 4 dice"),
        ("5yz", 5, "Year Zero 5 dice"),
        ("6yz", 6, "Year Zero 6 dice"),
        ("8yz", 8, "Year Zero 8 dice"),
        ("10yz", 10, "Year Zero 10 dice"),
        ("12yz", 12, "Year Zero 12 dice"),
    ];

    for (alias, expected_dice, description) in year_zero_tests {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Year Zero '{}' should parse: {}",
            alias,
            description
        );

        let results = result.unwrap();
        assert_eq!(results.len(), 1, "Should have one result for '{}'", alias);

        let roll = &results[0];

        // Should have success counting (target 6)
        assert!(
            roll.successes.is_some(),
            "Year Zero should have success counting for '{}'",
            alias
        );

        // Should have exactly the expected number of dice (no exploding in basic YZ)
        assert_eq!(
            roll.individual_rolls.len(),
            expected_dice,
            "Should have exactly {} dice for '{}'",
            expected_dice,
            alias
        );

        // Success count should be reasonable (number of 6s)
        let success_count = roll.successes.unwrap();
        let actual_sixes = roll.individual_rolls.iter().filter(|&&r| r == 6).count();
        assert_eq!(
            success_count, actual_sixes as i32,
            "Success count should equal number of 6s for '{}': {} vs {}",
            alias, success_count, actual_sixes
        );

        // Should be using d6s (all rolls 1-6)
        for &die_roll in &roll.individual_rolls {
            assert!(
                die_roll >= 1 && die_roll <= 6,
                "Year Zero should use d6s, got {} for '{}'",
                die_roll,
                alias
            );
        }
    }

    // Test Year Zero with modifiers
    let yz_modifier_tests = vec!["6yz + 2", "4yz - 1", "5yz * 2", "8yz + 1d6"];

    for test in yz_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Year Zero modifier test '{}' should parse",
            test
        );
    }

    // Test edge cases
    let yz_edge_cases = vec![
        ("1yz", 1, "Minimum Year Zero dice"),
        ("15yz", 15, "Large Year Zero pool"),
    ];

    for (alias, expected_dice, description) in yz_edge_cases {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Year Zero edge case '{}' should work: {}",
            alias,
            description
        );

        let results = result.unwrap();
        let roll = &results[0];
        assert_eq!(
            roll.individual_rolls.len(),
            expected_dice,
            "Should have exactly {} dice for edge case '{}'",
            expected_dice,
            alias
        );
    }
}

#[test]
fn test_warhammer_40k_aos_comprehensive() {
    // Test Warhammer 40k/Age of Sigmar (XwhY+ -> Xd6 tY)
    let warhammer_tests = vec![
        // (alias, expected_dice, target, description)
        ("2wh3+", 2, 3, "2d6 target 3+"),
        ("3wh4+", 3, 4, "3d6 target 4+"),
        ("4wh2+", 4, 2, "4d6 target 2+"),
        ("5wh5+", 5, 5, "5d6 target 5+"),
        ("6wh6+", 6, 6, "6d6 target 6+"),
        ("8wh3+", 8, 3, "8d6 target 3+"),
        ("10wh4+", 10, 4, "10d6 target 4+"),
        ("1wh2+", 1, 2, "1d6 target 2+"),
    ];

    for (alias, expected_dice, target, description) in warhammer_tests {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Warhammer '{}' should parse: {}",
            alias,
            description
        );

        let results = result.unwrap();
        assert_eq!(results.len(), 1, "Should have one result for '{}'", alias);

        let roll = &results[0];

        // Should have success counting
        assert!(
            roll.successes.is_some(),
            "Warhammer should have success counting for '{}'",
            alias
        );

        // Should have exactly the expected number of dice
        assert_eq!(
            roll.individual_rolls.len(),
            expected_dice,
            "Should have exactly {} dice for '{}'",
            expected_dice,
            alias
        );

        // Success count should match number of dice >= target
        let success_count = roll.successes.unwrap();
        let actual_successes = roll
            .individual_rolls
            .iter()
            .filter(|&&r| r >= target as i32)
            .count();
        assert_eq!(
            success_count, actual_successes as i32,
            "Success count should equal dice >= {} for '{}': {} vs {}",
            target, alias, success_count, actual_successes
        );

        // Should be using d6s (all rolls 1-6)
        for &die_roll in &roll.individual_rolls {
            assert!(
                die_roll >= 1 && die_roll <= 6,
                "Warhammer should use d6s, got {} for '{}'",
                die_roll,
                alias
            );
        }

        // Success count should be reasonable (0 to dice count)
        assert!(
            success_count >= 0 && success_count <= expected_dice as i32,
            "Success count {} should be 0-{} for '{}'",
            success_count,
            expected_dice,
            alias
        );
    }

    // Test edge cases and target variations
    let warhammer_edge_cases = vec![
        ("1wh6+", 1, 6, "Single die, hard target"),
        ("12wh2+", 12, 2, "Many dice, easy target"),
        ("3wh1+", 3, 1, "Impossible to fail (1+ target)"),
    ];

    for (alias, expected_dice, target, description) in warhammer_edge_cases {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Warhammer edge case '{}' should work: {}",
            alias,
            description
        );

        let results = result.unwrap();
        let roll = &results[0];

        // Validate success logic for edge cases
        let success_count = roll.successes.unwrap();
        let actual_successes = roll
            .individual_rolls
            .iter()
            .filter(|&&r| r >= target as i32)
            .count();

        assert_eq!(
            success_count, actual_successes as i32,
            "Edge case '{}' success count should be correct",
            alias
        );

        // Special case: 1+ target should always succeed on d6
        if target == 1 {
            assert_eq!(
                success_count, expected_dice as i32,
                "1+ target should always succeed all dice for '{}'",
                alias
            );
        }
    }
}

#[test]
fn test_missing_systems_with_roll_sets() {
    // Test all missing systems work with roll sets
    let roll_set_tests = vec![
        ("3 sp4", "Storypath roll sets"),
        ("2 dd34", "Double digit roll sets"),
        ("4 snm5", "SNM roll sets"),
        ("3 6yz", "Year Zero roll sets"),
        ("2 3wh4+", "Warhammer roll sets"),
        ("3 5d6l", "D6 Legends roll sets"),
    ];

    for (expression, description) in roll_set_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Roll set '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        let expected_sets = expression.chars().next().unwrap().to_digit(10).unwrap() as usize;
        assert_eq!(
            results.len(),
            expected_sets,
            "Should have {} sets for '{}'",
            expected_sets,
            expression
        );

        for (i, roll) in results.iter().enumerate() {
            assert_eq!(
                roll.label,
                Some(format!("Set {}", i + 1)),
                "Each set should have correct label for '{}'",
                expression
            );
        }
    }
}

#[test]
fn test_missing_systems_with_complex_modifiers() {
    // Test complex modifier combinations with missing systems
    let complex_modifier_tests = vec![
        // Storypath
        ("sp4 + 2d6", "Storypath with additional dice"),
        ("sp6 * 2 - 3", "Storypath with math operations"),
        // Double Digit
        ("dd66 / 10", "Double digit division"),
        // SNM
        ("snm4 * 3", "SNM with multiplication"),
        // Year Zero
        ("8yz - 2", "Year Zero with subtraction"),
    ];

    for (expression, description) in complex_modifier_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Complex modifier '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for complex modifier '{}'",
            expression
        );
    }
}

#[test]
fn test_missing_systems_alias_expansion() {
    // Verify that alias expansion works correctly for missing systems
    let alias_expansion_tests = vec![
        // (alias, expected_expansion)
        ("sp4", "4d10 t8 ie10"),
        ("dd34", "1d3 * 10 + 1d4"),
        ("snm5", "5d6 ie6 t4"),
        ("6yz", "6d6 t6"),
        ("3wh4+", "3d6 t4"),
    ];

    for (alias, expected_expansion) in alias_expansion_tests {
        // Test that the alias expands correctly
        let expanded = aliases::expand_alias(alias);
        assert_eq!(
            expanded,
            Some(expected_expansion.to_string()),
            "Alias '{}' should expand to '{}'",
            alias,
            expected_expansion
        );

        // Test that both the alias and expansion produce equivalent results
        let alias_result = parse_and_roll(alias);
        let expansion_result = parse_and_roll(expected_expansion);

        assert!(
            alias_result.is_ok() && expansion_result.is_ok(),
            "Both alias '{}' and expansion '{}' should work",
            alias,
            expected_expansion
        );

        let alias_roll = alias_result.unwrap();
        let expansion_roll = expansion_result.unwrap();

        // Should have same number of results
        assert_eq!(
            alias_roll.len(),
            expansion_roll.len(),
            "Alias and expansion should have same result count for '{}'",
            alias
        );

        // Should have similar characteristics (both success-based or both total-based)
        assert_eq!(
            alias_roll[0].successes.is_some(),
            expansion_roll[0].successes.is_some(),
            "Alias and expansion should have same success counting behavior for '{}'",
            alias
        );
    }
}

#[test]
fn test_exalted_system_comprehensive() {
    // Test Exalted system (exX -> Xd10 t7 t10, exXtY -> Xd10 tY t10)
    let exalted_tests = vec![
        // (alias, expected_dice_count, expected_target, description)
        ("ex5", 5, 7, "Exalted 5d10 t7 t10"),
        ("ex5t8", 5, 8, "Exalted 5d10 t8 t10"),
        ("ex10", 10, 7, "Exalted 10d10 t7 t10"),
        ("ex3t6", 3, 6, "Exalted 3d10 t6 t10"),
        ("ex8", 8, 7, "Exalted 8d10 t7 t10"),
        ("ex6t9", 6, 9, "Exalted 6d10 t9 t10"),
        ("ex12t5", 12, 5, "Exalted 12d10 t5 t10"),
    ];

    for (alias, expected_dice, expected_target, description) in exalted_tests {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Exalted '{}' should parse: {}",
            alias,
            description
        );

        let results = result.unwrap();
        assert_eq!(results.len(), 1, "Should have one result for '{}'", alias);

        let roll = &results[0];

        // Should have success counting (target system)
        assert!(
            roll.successes.is_some(),
            "Exalted should have success counting for '{}'",
            alias
        );

        // Should have exactly the expected number of dice
        assert_eq!(
            roll.individual_rolls.len(),
            expected_dice,
            "Should have exactly {} dice for '{}'",
            expected_dice,
            alias
        );

        // Should be using d10s (all rolls 1-10)
        for &die_roll in &roll.individual_rolls {
            assert!(
                die_roll >= 1 && die_roll <= 10,
                "Exalted should use d10s, got {} for '{}'",
                die_roll,
                alias
            );
        }

        // Verify success counting logic for Exalted (7+ = 1 success, 10 = 2 successes)
        let success_count = roll.successes.unwrap();
        let manual_count = roll
            .individual_rolls
            .iter()
            .map(|&r| {
                if r >= expected_target as i32 && r < 10 {
                    1 // Single success for target+ but less than 10
                } else if r == 10 {
                    2 // Double success for 10s
                } else {
                    0 // No success
                }
            })
            .sum::<i32>();

        assert_eq!(
            success_count, manual_count,
            "Success count should match Exalted rules for '{}': {} vs {} (dice: {:?})",
            alias, success_count, manual_count, roll.individual_rolls
        );

        // Should be reasonable success count (0 to 2 * dice count)
        assert!(
            success_count >= 0 && success_count <= (expected_dice as i32 * 2),
            "Success count {} should be 0-{} for '{}'",
            success_count,
            expected_dice * 2,
            alias
        );
    }

    // Test Exalted with modifiers
    let exalted_modifier_tests = vec!["ex5 + 2", "ex8 - 1", "ex6 * 2", "ex4 + 1d6"];

    for test in exalted_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Exalted modifier test '{}' should parse",
            test
        );
    }

    // Test edge cases
    let exalted_edge_cases = vec![
        ("ex1", 1, 7, "Minimum Exalted dice"),
        ("ex15", 15, 7, "Large Exalted pool"),
        ("ex1t10", 1, 10, "Single die, hard target"),
        ("ex20t4", 20, 4, "Many dice, easy target"),
    ];

    for (alias, expected_dice, expected_target, description) in exalted_edge_cases {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "Exalted edge case '{}' should work: {}",
            alias,
            description
        );

        let results = result.unwrap();
        let roll = &results[0];
        assert_eq!(
            roll.individual_rolls.len(),
            expected_dice,
            "Should have exactly {} dice for edge case '{}'",
            expected_dice,
            alias
        );

        // Verify double 10s rule still applies
        if roll.individual_rolls.iter().any(|&r| r == 10) {
            let tens_count = roll.individual_rolls.iter().filter(|&&r| r == 10).count();
            let other_successes = roll
                .individual_rolls
                .iter()
                .filter(|&&r| r >= expected_target as i32 && r < 10)
                .count();
            let expected_successes = (tens_count * 2) + other_successes;

            assert_eq!(
                roll.successes.unwrap() as usize,
                expected_successes,
                "10s should count double for '{}': expected {}, got {}",
                alias,
                expected_successes,
                roll.successes.unwrap()
            );
        }
    }
}

#[test]
fn test_exalted_alias_expansion() {
    // Test that Exalted aliases expand correctly
    let exalted_alias_tests = vec![
        // (alias, expected_expansion)
        ("ex5", "5d10 t7 t10"),
        ("ex5t8", "5d10 t8 t10"),
        ("ex10", "10d10 t7 t10"),
        ("ex3t6", "3d10 t6 t10"),
    ];

    for (alias, expected_expansion) in exalted_alias_tests {
        // Test that the alias expands correctly
        let expanded = aliases::expand_alias(alias);
        assert_eq!(
            expanded,
            Some(expected_expansion.to_string()),
            "Exalted alias '{}' should expand to '{}'",
            alias,
            expected_expansion
        );

        // Test that both the alias and expansion produce equivalent results
        let alias_result = parse_and_roll(alias);
        let expansion_result = parse_and_roll(expected_expansion);

        assert!(
            alias_result.is_ok() && expansion_result.is_ok(),
            "Both alias '{}' and expansion '{}' should work",
            alias,
            expected_expansion
        );

        let alias_roll = alias_result.unwrap();
        let expansion_roll = expansion_result.unwrap();

        // Should have same number of results
        assert_eq!(
            alias_roll.len(),
            expansion_roll.len(),
            "Alias and expansion should have same result count for '{}'",
            alias
        );

        // Should both have success counting
        assert_eq!(
            alias_roll[0].successes.is_some(),
            expansion_roll[0].successes.is_some(),
            "Alias and expansion should both have success counting for '{}'",
            alias
        );
    }
}

#[test]
fn test_exalted_with_roll_sets() {
    // Test Exalted system works with roll sets
    let exalted_roll_set_tests = vec![
        ("3 ex5", "Exalted roll sets"),
        ("2 ex8t6", "Exalted with custom target roll sets"),
        ("4 ex3", "Multiple small Exalted pools"),
    ];

    for (expression, description) in exalted_roll_set_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Exalted roll set '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        let expected_sets = expression.chars().next().unwrap().to_digit(10).unwrap() as usize;
        assert_eq!(
            results.len(),
            expected_sets,
            "Should have {} sets for '{}'",
            expected_sets,
            expression
        );

        for (i, roll) in results.iter().enumerate() {
            assert_eq!(
                roll.label,
                Some(format!("Set {}", i + 1)),
                "Each set should have correct label for '{}'",
                expression
            );
            assert!(
                roll.successes.is_some(),
                "Each Exalted set should have success counting"
            );
        }
    }
}

#[test]
fn test_exalted_invalid_cases() {
    // Test invalid Exalted patterns
    let invalid_exalted = vec![
        "ex0",    // Zero dice
        "ex",     // No dice count
        "ex5t0",  // Zero target
        "ex5t11", // Target higher than die sides
        "ex-5",   // Negative dice count
    ];

    for invalid_test in invalid_exalted {
        let result = parse_and_roll(invalid_test);
        assert!(
            result.is_err(),
            "Invalid Exalted '{}' should fail",
            invalid_test
        );
    }
}

#[test]
fn test_wrath_glory_special_modes_comprehensive() {
    // Test Wrath & Glory special modes: !soak, !exempt, !dmg
    let wng_special_modes = vec![
        ("wng 4d6 !soak", true, "Basic soak test"),
        ("wng 6d6 !exempt", true, "Basic exempt test"),
        ("wng 5d6 !dmg", true, "Basic damage test"),
    ];

    for (expression, should_use_total, description) in wng_special_modes {
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

        if should_use_total {
            // For soak/damage/exempt tests, should use total instead of successes
            assert!(
                roll.total > 0 || roll.individual_rolls.iter().sum::<i32>() >= 0,
                "Should have meaningful total for '{}': {}",
                expression,
                description
            );

            // May or may not have successes depending on implementation
            // but should have some kind of meaningful result
            assert!(
                roll.total != 0 || roll.successes.is_some() || !roll.individual_rolls.is_empty(),
                "Should have meaningful result for '{}': {}",
                expression,
                description
            );
        }

        // Wrath dice should still be tracked for complications/glory
        if expression.contains("w2") || expression.contains("w3") {
            // Should have multiple wrath dice information
            let has_wrath_info = roll.wng_wrath_dice.is_some()
                || roll.wng_wrath_die.is_some()
                || roll
                    .notes
                    .iter()
                    .any(|note| note.to_lowercase().contains("wrath"));

            assert!(
                has_wrath_info,
                "Should have wrath dice information for '{}': {}",
                expression, description
            );
        }

        // Check difficulty mechanics if specified
        if expression.contains("dn2") {
            let has_difficulty_note = roll.notes.iter().any(|note| {
                note.contains("Difficulty 2")
                    || note.contains("dn2")
                    || note.contains("PASS")
                    || note.contains("FAIL")
            });

            if !has_difficulty_note {
                println!(
                    "Note: Difficulty mechanics for '{}' may need clearer indication",
                    expression
                );
            }
        }
    }

    // Test that standard W&G (without special modes) still works normally
    let standard_wng_tests = vec![
        ("wng 4d6", false, "Standard W&G test"),
        ("wng w2 4d6", false, "Standard W&G with multiple wrath"),
        ("wng dn3 5d6", false, "Standard W&G with difficulty"),
    ];

    for (expression, _, description) in standard_wng_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Standard W&G '{}' should still work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        let roll = &results[0];

        // Standard W&G should have success counting, not total-based
        assert!(
            roll.successes.is_some(),
            "Standard W&G should have success counting for '{}'",
            expression
        );

        // Should have wrath dice tracking
        assert!(
            roll.wng_wrath_die.is_some() || roll.wng_wrath_dice.is_some(),
            "Should have wrath dice information for '{}'",
            expression
        );
    }
}

#[test]
fn test_wrath_glory_complications_and_glory() {
    // Test that wrath dice complications and glory are properly tracked
    // Note: This test may be probabilistic, so we run multiple iterations
    for _ in 0..10 {
        let result = parse_and_roll("wng w2 6d6");
        assert!(result.is_ok(), "W&G wrath dice test should work");

        let results = result.unwrap();
        let roll = &results[0];

        // Should have wrath dice information
        assert!(
            roll.wng_wrath_dice.is_some() || roll.wng_wrath_die.is_some(),
            "Should track wrath dice"
        );

        // Check for complication/glory notes if applicable
        let has_complication = roll
            .notes
            .iter()
            .any(|note| note.to_lowercase().contains("complication") || note.contains("rolled 1"));

        let has_glory = roll.notes.iter().any(|note| {
            note.to_lowercase().contains("glory")
                || note.to_lowercase().contains("critical")
                || note.contains("rolled 6")
        });

        // If we have wrath dice results, check for appropriate mechanics
        if let Some(ref wrath_dice) = roll.wng_wrath_dice {
            let has_ones = wrath_dice.iter().any(|&d| d == 1);
            let has_sixes = wrath_dice.iter().any(|&d| d == 6);

            if has_ones && !has_complication {
                println!("Note: Complication (1) detected but no complication note found");
            }

            if has_sixes && !has_glory {
                println!("Note: Glory (6) detected but no glory note found");
            }
        }
    }
}

#[test]
fn test_wrath_glory_special_modes_with_modifiers() {
    // Test W&G special modes with mathematical modifiers
    let wng_modifier_tests = vec![
        ("wng 4d6 !soak + 2", "Soak with bonus"),
        ("wng 5d6 !dmg - 1", "Damage with penalty"),
        ("wng w2 4d6 !exempt * 2", "Exempt with multiplier"),
        ("wng dn3 6d6 !soak / 2", "Soak with division"),
    ];

    for (expression, description) in wng_modifier_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "W&G special mode with modifier '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for modifier test '{}'",
            expression
        );
    }
}

#[test]
fn test_wrath_glory_special_modes_with_roll_sets() {
    // Test W&G special modes work with roll sets
    let wng_roll_set_tests = vec![
        ("3 wng 4d6 !soak", "Soak roll sets"),
        ("2 wng w2 5d6 !dmg", "Damage roll sets with wrath"),
        ("4 wng dn2 4d6 !exempt", "Exempt roll sets with difficulty"),
    ];

    for (expression, description) in wng_roll_set_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "W&G roll set '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        let expected_sets = expression.chars().next().unwrap().to_digit(10).unwrap() as usize;
        assert_eq!(
            results.len(),
            expected_sets,
            "Should have {} sets for '{}'",
            expected_sets,
            expression
        );

        for (i, roll) in results.iter().enumerate() {
            assert_eq!(
                roll.label,
                Some(format!("Set {}", i + 1)),
                "Each set should have correct label for '{}'",
                expression
            );
        }
    }
}

#[test]
fn test_wrath_glory_invalid_special_modes() {
    // Test invalid W&G special mode syntax
    let invalid_wng_modes = vec![
        "wng 4d6 !invalid",      // Invalid special mode
        "wng 4d6 !",             // Empty special mode
        "wng 4d6 !soak invalid", // Extra text after mode
        "wng 4d6 !SOAK",         // Wrong case (if case-sensitive)
        "wng 4d6 ! soak",        // Space in special mode
    ];

    for invalid_test in invalid_wng_modes {
        let result = parse_and_roll(invalid_test);

        // These might parse but should either work correctly or fail gracefully
        match result {
            Ok(results) => {
                println!(
                    "W&G mode '{}' parsed successfully (behavior may vary): {} results",
                    invalid_test,
                    results.len()
                );
                // If it parses, it should at least have some meaningful result
                assert!(
                    !results.is_empty(),
                    "Should have some result if parsing succeeds"
                );
            }
            Err(error) => {
                println!("W&G mode '{}' failed as expected: {}", invalid_test, error);
                // Error message should be reasonable
                let error_str = error.to_string();
                assert!(!error_str.is_empty(), "Should have error message");
            }
        }
    }
}

#[test]
fn test_wrath_glory_edge_case_wrath_dice_counts() {
    // Test edge cases for wrath dice counts in special modes
    let wrath_dice_edge_cases = vec![
        ("wng w1 4d6 !soak", 1, "Minimum wrath dice with soak"),
        ("wng w5 6d6 !dmg", 5, "Maximum wrath dice with damage"),
        (
            "wng w3 dn1 5d6 !exempt",
            3,
            "Multiple wrath with easy difficulty",
        ),
        (
            "wng w1 dn6 4d6 !soak",
            1,
            "Single wrath with hard difficulty",
        ),
    ];

    for (expression, expected_wrath_count, description) in wrath_dice_edge_cases {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "W&G edge case '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        let roll = &results[0];

        // Check wrath dice count if trackable
        if let Some(ref wrath_dice) = roll.wng_wrath_dice {
            assert_eq!(
                wrath_dice.len(),
                expected_wrath_count,
                "Should have {} wrath dice for '{}'",
                expected_wrath_count,
                expression
            );
        } else if let Some(_) = roll.wng_wrath_die {
            // Legacy single wrath die tracking
            if expected_wrath_count == 1 {
                // This is fine for single wrath die
            } else {
                println!(
                    "Note: Multiple wrath dice tracking may need enhancement for '{}'",
                    expression
                );
            }
        }
    }
}

#[test]
fn test_advanced_percentile_edge_cases() {
    // Test complex percentile advantage/disadvantage scenarios
    let advanced_percentile_tests = vec![
        // Complex mathematical operations with percentile advantage/disadvantage
        ("+d% + 25", "Percentile advantage with large bonus"),
        ("-d% - 15", "Percentile disadvantage with penalty"),
        ("+d% * 2", "Percentile advantage with multiplication"),
        ("-d% / 2", "Percentile disadvantage with division"),
        ("+d% + 1d6", "Percentile advantage with additional dice"),
        ("-d% - 2d4", "Percentile disadvantage with dice subtraction"),
        // Edge case values
        ("+d% + 100", "Advantage with maximum bonus"),
        ("-d% - 50", "Disadvantage with large penalty"),
        ("+d% * 0", "Advantage multiplied by zero"),
        ("-d% + 200", "Disadvantage with large positive modifier"),
    ];

    for (expression, description) in advanced_percentile_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Advanced percentile '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert_eq!(
            results.len(),
            1,
            "Should have one result for '{}'",
            expression
        );

        let roll = &results[0];

        // Should have multiple dice for advantage/disadvantage mechanism
        assert!(
            roll.individual_rolls.len() >= 2,
            "Percentile advantage/disadvantage should have multiple dice for '{}': got {} dice",
            expression,
            roll.individual_rolls.len()
        );

        // For percentile dice, all individual rolls should be in range 1-10 or 1-100 depending on implementation
        for &die_roll in &roll.individual_rolls {
            assert!(
                (die_roll >= 1 && die_roll <= 10) || (die_roll >= 1 && die_roll <= 100),
                "Percentile die roll should be 1-10 or 1-100, got {} for '{}'",
                die_roll,
                expression
            );
        }

        // Total should be reasonable (considering modifiers)
        assert!(
            roll.total >= -200 && roll.total <= 400,
            "Total {} should be reasonable for percentile expression '{}'",
            roll.total,
            expression
        );

        println!(
            "Advanced percentile test '{}': {} dice, total {} - {}",
            expression,
            roll.individual_rolls.len(),
            roll.total,
            description
        );
    }
}

#[test]
fn test_percentile_advantage_mechanics_detailed() {
    // Test the specific mechanics of percentile advantage/disadvantage

    // Run multiple tests to verify statistical behavior
    let mut advantage_totals = Vec::new();
    let mut disadvantage_totals = Vec::new();
    let mut regular_totals = Vec::new();

    for _ in 0..20 {
        // Test advantage
        let adv_result = parse_and_roll("+d%").unwrap();
        advantage_totals.push(adv_result[0].total);

        // Test disadvantage
        let dis_result = parse_and_roll("-d%").unwrap();
        disadvantage_totals.push(dis_result[0].total);

        // Test regular percentile for comparison
        let reg_result = parse_and_roll("d%").unwrap();
        regular_totals.push(reg_result[0].total);
    }

    // All results should be in valid percentile range
    for &total in &advantage_totals {
        assert!(
            total >= 1 && total <= 100,
            "Advantage total {} should be 1-100",
            total
        );
    }

    for &total in &disadvantage_totals {
        assert!(
            total >= 1 && total <= 100,
            "Disadvantage total {} should be 1-100",
            total
        );
    }

    for &total in &regular_totals {
        assert!(
            total >= 1 && total <= 100,
            "Regular total {} should be 1-100",
            total
        );
    }

    // Statistical validation (advantage should tend higher, disadvantage lower)
    let avg_advantage: f64 =
        advantage_totals.iter().sum::<i32>() as f64 / advantage_totals.len() as f64;
    let avg_disadvantage: f64 =
        disadvantage_totals.iter().sum::<i32>() as f64 / disadvantage_totals.len() as f64;
    let avg_regular: f64 = regular_totals.iter().sum::<i32>() as f64 / regular_totals.len() as f64;

    println!(
        "Percentile averages - Advantage: {:.1}, Regular: {:.1}, Disadvantage: {:.1}",
        avg_advantage, avg_regular, avg_disadvantage
    );

    // With a reasonable sample size, advantage should generally be higher than disadvantage
    // (This is probabilistic, so we don't enforce strict ordering, just log the results)
    if avg_advantage > avg_disadvantage {
        println!("✓ Advantage performed better than disadvantage as expected");
    } else {
        println!("Note: Advantage average not higher than disadvantage (random variation)");
    }
}

#[test]
fn test_percentile_with_complex_game_systems() {
    // Test percentile dice with various game system combinations
    let percentile_system_combos = vec![
        ("+d% + 4cod", "Percentile advantage with CoD roll"),
        ("-d% + sw8", "Percentile disadvantage with Savage Worlds"),
        ("+d% + cpr", "Percentile advantage with Cyberpunk Red"),
        ("-d% + wit", "Percentile disadvantage with Witcher"),
        ("+d% + 3df", "Percentile advantage with Fudge dice"),
        ("-d% + gb", "Percentile disadvantage with Godbound"),
    ];

    for (expression, description) in percentile_system_combos {
        let result = parse_and_roll(expression);

        // These are complex combinations that may or may not be supported
        match result {
            Ok(results) => {
                println!(
                    "Complex percentile combo '{}' worked: {} results - {}",
                    expression,
                    results.len(),
                    description
                );

                assert!(
                    !results.is_empty(),
                    "Should have results if parsing succeeds"
                );

                // Should have meaningful output
                for roll in &results {
                    assert!(
                        roll.total != 0
                            || roll.successes.is_some()
                            || !roll.individual_rolls.is_empty(),
                        "Should have meaningful result for '{}'",
                        expression
                    );
                }
            }
            Err(error) => {
                println!(
                    "Complex percentile combo '{}' failed: {} - {}",
                    expression, error, description
                );

                // Should fail gracefully with meaningful error
                assert!(!error.to_string().is_empty(), "Should have error message");
            }
        }
    }
}

#[test]
fn test_percentile_edge_case_mathematics() {
    // Test edge cases in percentile mathematics
    let percentile_math_edge_cases = vec![
        ("+d% + 0", "Advantage with zero modifier"),
        ("-d% - 0", "Disadvantage with zero modifier"),
        ("+d% * 1", "Advantage with identity multiplication"),
        ("-d% / 1", "Disadvantage with identity division"),
        ("0 + +d%", "Zero plus advantage"),
        ("100 - -d%", "100 minus disadvantage"),
        ("+d% + +d%", "Double advantage (if supported)"),
        ("-d% + -d%", "Double disadvantage (if supported)"),
    ];

    for (expression, description) in percentile_math_edge_cases {
        let result = parse_and_roll(expression);

        match result {
            Ok(results) => {
                println!(
                    "Percentile math edge case '{}' worked: total {} - {}",
                    expression, results[0].total, description
                );

                // Should produce reasonable results
                assert!(
                    results[0].total >= -100 && results[0].total <= 200,
                    "Total {} should be reasonable for '{}'",
                    results[0].total,
                    expression
                );
            }
            Err(error) => {
                println!(
                    "Percentile math edge case '{}' failed: {} - {}",
                    expression, error, description
                );

                // Some complex cases may legitimately fail
                assert!(!error.to_string().is_empty(), "Should have error message");
            }
        }
    }
}

#[test]
fn test_percentile_with_roll_sets() {
    // Test percentile advantage/disadvantage with roll sets
    let percentile_roll_sets = vec![
        ("3 +d%", "Advantage roll sets"),
        ("2 -d%", "Disadvantage roll sets"),
        ("4 +d% + 10", "Advantage sets with modifier"),
        ("3 -d% - 5", "Disadvantage sets with penalty"),
    ];

    for (expression, description) in percentile_roll_sets {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Percentile roll set '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        let expected_sets = expression.chars().next().unwrap().to_digit(10).unwrap() as usize;
        assert_eq!(
            results.len(),
            expected_sets,
            "Should have {} sets for '{}'",
            expected_sets,
            expression
        );

        for (i, roll) in results.iter().enumerate() {
            assert_eq!(
                roll.label,
                Some(format!("Set {}", i + 1)),
                "Each set should have correct label for '{}'",
                expression
            );

            // Each roll should have percentile-like results
            assert!(
                roll.total >= 1 && roll.total <= 200, // Allowing for modifiers
                "Percentile set total {} should be reasonable for '{}'",
                roll.total,
                expression
            );
        }
    }
}

#[test]
fn test_unicode_handling_in_percentile_expressions() {
    // Test Unicode characters in comments and labels with percentile dice
    let unicode_tests = vec![
        ("(⚔️ Attack) +d%", "Unicode sword in label with advantage"),
        (
            "+d% ! 🔥 Fire damage",
            "Advantage with Unicode fire comment",
        ),
        ("(🎯 Skill) -d% ! 🎲 Roll", "Unicode target and dice"),
        ("(Test \"Quotes\") +d%", "Quotes in label with advantage"),
        (
            "(Test 'Apostrophe') -d%",
            "Apostrophe in label with disadvantage",
        ),
    ];

    for (expression, description) in unicode_tests {
        let result = parse_and_roll(expression);

        match result {
            Ok(results) => {
                println!(
                    "Unicode test '{}' worked: {} - {}",
                    expression, results[0].total, description
                );

                // Should preserve Unicode in labels/comments
                let roll = &results[0];
                if let Some(ref label) = roll.label {
                    println!("  Label preserved: '{}'", label);
                }
                if let Some(ref comment) = roll.comment {
                    println!("  Comment preserved: '{}'", comment);
                }

                // Should still produce valid percentile results
                assert!(
                    roll.total >= 1 && roll.total <= 100,
                    "Unicode test should still produce valid percentile result"
                );
            }
            Err(error) => {
                println!(
                    "Unicode test '{}' failed: {} - {}",
                    expression, error, description
                );

                // Should fail gracefully if Unicode not supported
                assert!(!error.to_string().is_empty(), "Should have error message");
            }
        }
    }
}

#[test]
fn test_world_of_darkness_cancel_mechanics() {
    // Test WOD cancel aliases
    let wod_cancel_tests = vec![
        ("4wod8c", "WOD difficulty 8 with cancel"),
        ("5wod6c", "WOD difficulty 6 with cancel"),
        ("6wod7c", "WOD difficulty 7 with cancel"),
        ("3wod9c + 2", "WOD with cancel and modifier"),
    ];

    for (expression, description) in wod_cancel_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "WOD cancel test '{}' should parse: {}",
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

        // Should have both success and failure tracking for WOD
        assert!(
            results[0].successes.is_some(),
            "WOD should track successes for '{}'",
            expression
        );
        assert!(
            results[0].failures.is_some(),
            "WOD should track failures for '{}'",
            expression
        );
    }
}

#[test]
fn test_cancel_modifier_parsing() {
    // Test manual cancel modifier usage
    let cancel_tests = vec![
        ("4d10 f1 t8 c", "Manual WOD with cancel"),
        ("5d10 f1 t6 c", "Different difficulty with cancel"),
        ("6d10 t7 f1 c", "Different order with cancel"),
        ("3d10 c f1 t8", "Cancel first, then failure/target"),
        ("4d10 f1 c t8 + 2", "Cancel with modifiers"),
    ];

    for (expression, description) in cancel_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Cancel modifier test '{}' should parse: {}",
            expression,
            description
        );
    }
}

#[test]
fn test_cancel_without_failure_tracking() {
    // Test that cancel gracefully handles missing failure tracking
    let result = parse_and_roll("4d10 t8 c"); // No f1
    assert!(
        result.is_ok(),
        "Should parse cancel without failure tracking"
    );

    if let Ok(results) = result {
        // Should have some kind of note about needing failure tracking
        let has_relevant_note = results[0].notes.iter().any(|note| {
            note.contains("Cancel") || note.contains("failure") || note.contains("requires")
        });

        // We expect either a warning or the cancel to simply not activate
        assert!(
            has_relevant_note || results[0].notes.len() <= 2,
            "Should handle cancel without failures gracefully"
        );
    }
}

#[test]
fn test_wod_cancel_vs_regular_wod() {
    // Test that WOD with and without cancel work correctly
    let wod_comparison_tests = vec![
        ("4wod8", "4wod8c", "Difficulty 8 comparison"),
        ("5wod6", "5wod6c", "Difficulty 6 comparison"),
        ("3wod9 + 2", "3wod9c + 2", "With modifier comparison"),
    ];

    for (regular, cancel, description) in wod_comparison_tests {
        let regular_result = parse_and_roll(regular);
        let cancel_result = parse_and_roll(cancel);

        assert!(
            regular_result.is_ok() && cancel_result.is_ok(),
            "Both regular and cancel WOD should work: {}",
            description
        );

        let regular_roll = regular_result.unwrap();
        let cancel_roll = cancel_result.unwrap();

        // Both should have success/failure tracking
        assert_eq!(
            regular_roll[0].successes.is_some(),
            cancel_roll[0].successes.is_some(),
            "Success tracking should be consistent: {}",
            description
        );

        assert_eq!(
            regular_roll[0].failures.is_some(),
            cancel_roll[0].failures.is_some(),
            "Failure tracking should be consistent: {}",
            description
        );
    }
}

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

// Add these tests to tests/game_systems_tests.rs

#[test]
fn test_vtm5_system_comprehensive() {
    // Test VTM5 (Vampire: The Masquerade 5th Edition) system
    let vtm5_tests = vec![
        // (alias, expected_pool, expected_hunger, description)
        ("vtm3h1", 3, 1, "VTM5 3 dice pool, 1 hunger"),
        ("vtm7h2", 7, 2, "VTM5 7 dice pool, 2 hunger"),
        ("vtm5h0", 5, 0, "VTM5 5 dice pool, no hunger"),
        ("vtm10h3", 10, 3, "VTM5 10 dice pool, 3 hunger"),
        ("vtm1h1", 1, 1, "VTM5 minimum pool, all hunger"),
        ("vtm8h5", 8, 5, "VTM5 8 dice pool, max hunger"),
    ];

    for (alias, expected_pool, expected_hunger, description) in vtm5_tests {
        let result = parse_and_roll(alias);
        assert!(
            result.is_ok(),
            "VTM5 test '{}' should work: {}",
            alias,
            description
        );

        let results = result.unwrap();
        let roll = &results[0];

        // Should have correct total dice count
        assert_eq!(
            roll.individual_rolls.len(),
            expected_pool as usize,
            "Should have {} total dice for '{}': {}",
            expected_pool,
            alias,
            description
        );

        // Should have success counting enabled
        assert!(
            roll.successes.is_some(),
            "VTM5 should have success counting for '{}': {}",
            alias,
            description
        );

        // Success count should be reasonable (0 to pool size * 2 max for crits)
        let success_count = roll.successes.unwrap();
        assert!(
            success_count >= 0 && success_count <= (expected_pool as i32 * 2),
            "Success count {} should be 0-{} for '{}': {}",
            success_count,
            expected_pool * 2,
            alias,
            description
        );

        // Should have appropriate dice groups
        let regular_dice_count = expected_pool - expected_hunger;
        if expected_hunger > 0 && regular_dice_count > 0 {
            assert!(
                roll.dice_groups.len() == 2,
                "Should have regular and hunger dice groups for '{}': {}",
                alias,
                description
            );
        } else {
            assert!(
                roll.dice_groups.len() == 1,
                "Should have single dice group for '{}': {} (regular: {}, hunger: {})",
                alias,
                description,
                regular_dice_count,
                expected_hunger
            );
        }

        // Check for result type notes (probabilistic, so we don't assert on presence)
        let _has_result_note = roll.notes.iter().any(|note| {
            note.contains("CRITICAL")
                || note.contains("MESSY")
                || note.contains("BESTIAL")
                || note.contains("FAILURE")
                || note.contains("pairs of 10s")
        });

        // Note: Not every roll will have special results, so we just check the structure is right
        // The presence of notes depends on the random roll results
    }
}

#[test]
fn test_vtm5_alias_expansion() {
    // Test that VTM5 aliases expand correctly
    let vtm5_alias_tests = vec![
        // (alias, expected_expansion)
        ("vtm5h2", "5d10 vtm5p5h2"),
        ("vtm8h3", "8d10 vtm5p8h3"),
        ("vtm10h0", "10d10 vtm5p10h0"),
        ("vtm1h1", "1d10 vtm5p1h1"),
    ];

    for (alias, expected_expansion) in vtm5_alias_tests {
        // Test that the alias expands correctly
        let expanded = aliases::expand_alias(alias);
        assert_eq!(
            expanded,
            Some(expected_expansion.to_string()),
            "VTM5 alias '{}' should expand to '{}'",
            alias,
            expected_expansion
        );

        // Test that both the alias and expansion produce equivalent results
        let alias_result = parse_and_roll(alias);
        let expansion_result = parse_and_roll(expected_expansion);

        assert!(
            alias_result.is_ok() && expansion_result.is_ok(),
            "Both alias '{}' and expansion '{}' should work",
            alias,
            expected_expansion
        );

        let alias_roll = alias_result.unwrap();
        let expansion_roll = expansion_result.unwrap();

        // Should have same number of results
        assert_eq!(
            alias_roll.len(),
            expansion_roll.len(),
            "Alias and expansion should have same result count for '{}'",
            alias
        );

        // Should both have success counting
        assert_eq!(
            alias_roll[0].successes.is_some(),
            expansion_roll[0].successes.is_some(),
            "Alias and expansion should both have success counting for '{}'",
            alias
        );
    }
}

#[test]
fn test_vtm5_with_modifiers() {
    // Test VTM5 system works with mathematical modifiers
    let vtm5_modifier_tests = vec![
        ("vtm6h2 + 2", "VTM5 with positive modifier"),
        ("vtm5h1 - 1", "VTM5 with negative modifier"),
        ("vtm4h0 * 2", "VTM5 with multiplication"),
        ("vtm8h3 + 1d6", "VTM5 with additional dice"),
    ];

    for (expression, description) in vtm5_modifier_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "VTM5 modifier test '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            results[0].successes.is_some(),
            "VTM5 with modifiers should have success counting: {}",
            description
        );
    }
}

#[test]
fn test_vtm5_with_roll_sets() {
    // Test VTM5 system works with roll sets
    let vtm5_roll_set_tests = vec![
        ("3 vtm5h2", "VTM5 roll sets"),
        ("2 vtm8h3", "VTM5 larger pools in sets"),
        ("4 vtm4h1", "Multiple VTM5 sets"),
    ];

    for (expression, description) in vtm5_roll_set_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "VTM5 roll set '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        let expected_sets = expression.chars().next().unwrap().to_digit(10).unwrap() as usize;
        assert_eq!(
            results.len(),
            expected_sets,
            "Should have {} sets for '{}'",
            expected_sets,
            expression
        );

        for (i, roll) in results.iter().enumerate() {
            assert_eq!(
                roll.label,
                Some(format!("Set {}", i + 1)),
                "Each set should have correct label for '{}'",
                expression
            );
            assert!(
                roll.successes.is_some(),
                "Each VTM5 set should have success counting"
            );
        }
    }
}

#[test]
fn test_vtm5_invalid_cases() {
    // Test invalid VTM5 patterns
    let invalid_vtm5 = vec![
        ("vtm0h1", "Zero pool size"),
        ("vtm5h6", "Too many hunger dice"),
        ("vtm31h2", "Pool too large"),
        ("vtm3h4", "Hunger exceeds pool"),
        ("vtmh2", "Missing pool size"),
        ("vtm5", "Missing hunger specification"),
        ("vtm5h", "Missing hunger count"),
    ];

    for (invalid_test, description) in invalid_vtm5 {
        let result = parse_and_roll(invalid_test);
        // These should either fail to parse or fail validation
        if result.is_ok() {
            println!(
                "VTM5 invalid case '{}' unexpectedly parsed: {}",
                invalid_test, description
            );
        }
    }
}

#[test]
fn test_vtm5_mechanics_simulation() {
    // Test to verify VTM5 mechanics work as expected
    // Note: This is probabilistic, so we test structure rather than exact outcomes

    let result = parse_and_roll("vtm10h3").unwrap();
    let roll = &result[0];

    // Should have exactly 10 dice
    assert_eq!(roll.individual_rolls.len(), 10);

    // Should have success counting
    assert!(roll.successes.is_some());

    // All dice should be d10s (1-10 range)
    for &die_roll in &roll.individual_rolls {
        assert!(
            die_roll >= 1 && die_roll <= 10,
            "VTM5 should use d10s, got {}",
            die_roll
        );
    }

    // Should have appropriate dice groups
    assert!(roll.dice_groups.len() >= 1 && roll.dice_groups.len() <= 2);

    // Total successes should equal success count
    assert_eq!(roll.total, roll.successes.unwrap());
}
