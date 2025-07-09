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
        "3 sw8", "3 cpr", "3 wit", "3 4cod", "3 gb", "3 mm", "3 cs 5",
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
