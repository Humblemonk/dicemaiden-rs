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
