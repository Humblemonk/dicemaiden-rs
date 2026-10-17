// tests/integration_tests.rs - End-to-End Integration Tests
//
// This file contains integration tests for complete dice rolling scenarios:
// - Multi-roll and roll set functionality
// - Discord formatting and message limits
// - Help topics
// - Comments, labels and flags
// - Error replies, /purge and bot-info formatting
//
// Whether an expression parses, and what it rolls, is pinned by the snapshot
// corpus (`tests/corpus/expressions.txt`); tests here check what the corpus
// cannot see, such as truncation and how features combine in the output.

use dicemaiden_rs::{
    commands::roll::error_reply, dice::parser, format_multiple_results,
    format_multiple_results_with_limit, help_text, parse_and_roll,
};

// ============================================================================
// ROLL SETS AND MULTIPLE ROLLS
// ============================================================================

/// Assert a roll-set expression formats as one line per labelled set followed by
/// a combined total.
fn assert_roll_set_formatting(expression: &str, expected_sets: usize) -> String {
    let results =
        parse_and_roll(expression).unwrap_or_else(|e| panic!("'{expression}' should roll: {e}"));
    let formatted = format_multiple_results(&results);

    for set in 1..=expected_sets {
        assert!(
            formatted.contains(&format!("Set {set}")),
            "'{expression}' output should name Set {set}"
        );
    }

    assert!(formatted.contains("**Total:"));

    let expected_total: i32 = results.iter().map(|r| r.total).sum();
    assert!(
        formatted.contains(&format!("Total: {expected_total}**")),
        "'{expression}' output should end on the combined total {expected_total}"
    );

    formatted
}

#[test]
fn test_roll_sets_comprehensive() {
    // Test roll set creation and labeling
    let result = parse_and_roll("6 4d6").unwrap();
    assert_eq!(result.len(), 6);

    for (i, roll) in result.iter().enumerate() {
        // Verify individual_rolls count since we can't access count directly
        assert!(
            roll.individual_rolls.len() >= 4,
            "Should have at least 4 dice (some may have exploded)"
        );
        assert_eq!(roll.label, Some(format!("Set {}", i + 1)));
        assert!(roll.total >= 4 && roll.total <= 24); // 4d6 range
    }
}

#[test]
fn test_roll_sets_with_complex_expressions() {
    // Test roll sets with mathematical operations
    let result = parse_and_roll("3 100/2d1").unwrap();
    assert_eq!(result.len(), 3);

    for roll in &result {
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        assert_eq!(roll.total, 50); // 100/2 = 50
    }

    // Test roll sets with game systems
    let result = parse_and_roll("4 4cod").unwrap();
    assert_eq!(result.len(), 4);

    for roll in &result {
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        assert!(roll.successes.is_some()); // CoD has success counting
    }
}

#[test]
fn test_semicolon_separated_rolls() {
    // Test multiple separate rolls with semicolons
    let result = parse_and_roll("1d20; 2d6; 1d4").unwrap();
    assert_eq!(result.len(), 3);

    // Each should have different dice configurations based on individual_rolls
    assert_eq!(result[0].individual_rolls.len(), 1); // 1d20
    assert_eq!(result[1].individual_rolls.len(), 2); // 2d6
    assert_eq!(result[2].individual_rolls.len(), 1); // 1d4

    // Should have original expressions stored
    assert!(result.iter().any(|r| r.original_expression.is_some()));
}

#[test]
fn test_roll_sets_vs_mathematical_expressions() {
    // Test disambiguation between roll sets and math

    // This should be 4 roll sets of (20/2d1)
    let result = parse_and_roll("4 20/2d1").unwrap();
    assert_eq!(result.len(), 4);
    for roll in &result {
        assert_eq!(roll.total, 10); // 20/2 = 10
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
    }

    // This should be single math expression (20 / 2d1)
    let result = parse_and_roll("20 / 2d1").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].total, 10);
    assert!(result[0].label.is_none());

    // This should be single math expression without spaces
    let result = parse_and_roll("20/2d1").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].total, 10);
    assert!(result[0].label.is_none());
}

#[test]
fn test_roll_set_validation_all_paths() {
    // A roll set holds 2 to 20 rolls on every parsing path: bare, behind a
    // flag, and in front of an alias. (expression, should succeed)
    let cases = vec![
        ("0 d6", false),
        ("1 d6", false),
        ("2 d6", true),
        ("20 d6", true),
        ("21 d6", false),
        ("22 d6", false),
        ("p 1 d6", false),
        ("p 3 d6", true),
        ("p 21 d6", false),
        ("s 21 d6", false),
        ("1 +d20", false),
        ("3 +d20", true),
        ("21 +d20", false),
    ];

    for (expression, should_succeed) in cases {
        match parse_and_roll(expression) {
            Ok(_) => assert!(should_succeed, "'{expression}' should be rejected"),
            Err(e) => {
                assert!(!should_succeed, "'{expression}' should roll: {e}");
                let message = e.to_string();
                assert!(
                    message.contains("Set count must be between 2 and 20")
                        || message.contains("Invalid set count"),
                    "'{expression}' should explain the set count: {message}"
                );
            }
        }
    }
}

// ============================================================================
// DISCORD FORMATTING AND LIMITS
// ============================================================================

#[test]
fn test_roll_set_formatting() {
    // Test roll set formatting with totals
    assert_roll_set_formatting("3 2d6", 3);
}

#[test]
fn test_semicolon_roll_formatting() {
    // Test semicolon-separated roll formatting
    let result = parse_and_roll("1d20; 2d6").unwrap();
    let formatted = format_multiple_results(&result);

    // Should show each request, without set labels or a combined total
    assert!(formatted.contains("Request:"));
    assert!(!formatted.contains("Set 1"));
    assert!(!formatted.contains("**Total:"));
}

#[test]
fn test_message_length_limits() {
    // Output must fit Discord's 2000 character limit, truncated with a notice
    // rather than failing the send, and still show the total
    let cases = vec![
        "20 50d6",
        "500d6",
        "500d1",
        "20 (Set) 10d6 e6 k5 ! Large roll set with exploding dice",
        "1d6 ie1",
        "20d10 t1 f10 b1",
    ];

    for expression in cases {
        let results = parse_and_roll(expression).unwrap();
        let formatted = format_multiple_results_with_limit(&results);

        assert!(
            formatted.len() <= 2000,
            "'{expression}' output is {} chars, over Discord's limit",
            formatted.len()
        );
        assert!(
            formatted.contains("**"),
            "'{expression}' output should still show a total: {formatted}"
        );
        assert!(
            !formatted.ends_with("..."),
            "'{expression}' output should end with a notice, not a bare ellipsis"
        );
    }
}

// ============================================================================
// HELP
// ============================================================================

#[test]
fn test_help_system_integration() {
    // Test that help text generation works
    let basic_help = help_text::generate_basic_help();
    assert!(basic_help.contains("Dice Maiden"));
    assert!(basic_help.contains("2d6 + 3d10"));
    assert!(basic_help.len() > 100);

    let alias_help = help_text::generate_alias_help();
    assert!(alias_help.contains("Game System Aliases"));
    assert!(alias_help.contains("Savage Worlds"));
    assert!(alias_help.len() > 100);

    let system_help = help_text::generate_system_help();
    assert!(system_help.contains("Game System Examples"));
    assert!(system_help.contains("Fudge/FATE"));
    assert!(system_help.len() > 100);
}

#[test]
fn test_help_topics_within_discord_message_limit() {
    // Help responses are sent verbatim, with none of the truncation that
    // format_multiple_results_with_limit applies to roll output, so a topic
    // that outgrows Discord's 2000 character cap fails to send entirely.
    // Discord counts UTF-16 code units, which is what the emoji and arrows
    // in these strings actually cost.
    for topic in help_text::HELP_TOPICS {
        let text = help_text::generate_topic_help(topic)
            .unwrap_or_else(|| panic!("HELP_TOPICS lists '{topic}' but it has no help text"));

        let units = text.encode_utf16().count();
        assert!(
            units <= 2000,
            "/help {topic} is {units} UTF-16 units, over Discord's 2000 limit — \
             split it into its own topic rather than growing this page"
        );
    }
}

#[test]
fn test_help_topic_dispatch_is_shared() {
    // Both /help <topic> and /roll help <topic> resolve through
    // generate_topic_help, so every advertised topic must resolve and no
    // surface can advertise a topic the other cannot serve.
    for topic in help_text::HELP_TOPICS {
        let text = help_text::generate_topic_help(topic)
            .unwrap_or_else(|| panic!("advertised topic '{topic}' does not resolve"));
        assert!(
            text.len() > 100,
            "topic '{topic}' resolved to suspiciously short text"
        );
    }

    // Unknown topics must not resolve — /roll relies on this to let
    // non-help input fall through to the dice parser
    for unknown in ["nonexistent", "", "OL", "help"] {
        assert!(
            help_text::generate_topic_help(unknown).is_none(),
            "'{unknown}' should not resolve as a help topic"
        );
    }

    assert!(
        help_text::HELP_TOPICS.contains(&"basic"),
        "basic is the documented default topic"
    );
    assert!(
        help_text::HELP_TOPICS.len() <= 25,
        "Discord allows at most 25 slash-command choices per option"
    );
}

#[test]
fn test_open_legend_help_topic() {
    let help = help_text::generate_open_legend_help();
    assert!(help.contains("Open Legend"));
    assert!(help.contains("ol5a2"));
    assert!(help.contains("adv#"));

    // Discoverable from the alias quick-reference
    assert!(
        help_text::generate_alias_help().contains("/help ol"),
        "alias help should point at the Open Legend topic"
    );
}

// ============================================================================
// COMMENTS, LABELS AND FLAGS
// ============================================================================

#[test]
fn test_comments_functionality() {
    // Test that comments are properly parsed, stored, and formatted
    let comment_scenarios = vec![
        ("2d6 ! Fire damage", "Fire damage"),
        (
            "1d20 + 5 ! Attack roll with sword",
            "Attack roll with sword",
        ),
        ("4d6 k3 ! Character creation", "Character creation"),
        ("8d6 ! Fireball spell", "Fireball spell"),
        ("3d10 t7 ! Difficulty check", "Difficulty check"),
        ("sw8 ! Savage Worlds trait test", "Savage Worlds trait test"),
        ("4cod ! Chronicles skill roll", "Chronicles skill roll"),
        // Advantage with math before the comment (issue #175)
        ("+d20+5 ! testing", "testing"),
        ("-d20+3 ! disadvantage roll", "disadvantage roll"),
        ("+d20-2 ! attack", "attack"),
    ];

    for (expression, expected_comment) in comment_scenarios {
        let result = parse_and_roll(expression).unwrap();

        // Verify comment is parsed correctly
        assert_eq!(
            result[0].comment,
            Some(expected_comment.to_string()),
            "Comment should be parsed correctly for '{}'",
            expression
        );

        // Verify comment appears in formatted output
        let formatted = format_multiple_results(&result);
        assert!(
            formatted.contains("Reason:"),
            "Formatted output should contain 'Reason:' for '{}'",
            expression
        );
        assert!(
            formatted.contains(expected_comment),
            "Formatted output should contain comment text for '{}'",
            expression
        );
    }

    // Test comment with roll sets
    let result = parse_and_roll("3 2d6 ! Damage rolls").unwrap();
    assert_eq!(result.len(), 3);

    // Only the first roll should have the comment to avoid duplication
    assert_eq!(result[0].comment, Some("Damage rolls".to_string()));

    let formatted = format_multiple_results(&result);
    // Should show comment once at the end, not for each set
    let comment_count = formatted.matches("Reason:").count();
    assert_eq!(
        comment_count, 1,
        "Comment should appear once in roll set output"
    );

    // Test comment with semicolon-separated rolls
    let result = parse_and_roll("1d20 ! Attack; 1d8 ! Damage").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].comment, Some("Attack".to_string()));
    assert_eq!(result[1].comment, Some("Damage".to_string()));

    // Test edge cases
    let edge_cases = vec![
        ("1d6 !", ""),     // Empty comment
        ("2d6 !    ", ""), // Whitespace-only comment
    ];

    for (expression, expected) in edge_cases {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(
            result[0].comment,
            Some(expected.to_string()),
            "Edge case comment for '{}'",
            expression
        );
    }
}

#[test]
fn test_labels_functionality() {
    // Test that labels are properly parsed, stored, and formatted
    let label_scenarios = vec![
        ("(Attack) 1d20 + 5", "Attack"),
        ("(Damage) 2d6", "Damage"),
        ("(Initiative) 1d20", "Initiative"),
        ("(Stealth Check) 1d20 - 2", "Stealth Check"),
        ("(Fire Damage) 8d6", "Fire Damage"),
        ("(Skill Test) 4cod", "Skill Test"),
        ("(Trait Roll) sw8", "Trait Roll"),
    ];

    for (expression, expected_label) in label_scenarios {
        let result = parse_and_roll(expression).unwrap();

        // Verify label is parsed correctly
        assert_eq!(
            result[0].label,
            Some(expected_label.to_string()),
            "Label should be parsed correctly for '{}'",
            expression
        );

        // Verify label appears in formatted output
        let formatted = format_multiple_results(&result);
        assert!(
            formatted.contains(&format!("**{}**:", expected_label)),
            "Formatted output should contain labeled format for '{}'",
            expression
        );
    }

    // Test label with roll sets
    let result = parse_and_roll("3 (Stat Roll) 4d6 k3").unwrap();
    assert_eq!(result.len(), 3);

    // Each roll in the set should have "Set X" label, overriding the original
    for (i, roll) in result.iter().enumerate() {
        assert_eq!(roll.label, Some(format!("Set {}", i + 1)));
    }

    // Test label + comment combination
    let result = parse_and_roll("(Attack) 1d20 + 5 ! With magic sword").unwrap();
    assert_eq!(result[0].label, Some("Attack".to_string()));
    assert_eq!(result[0].comment, Some("With magic sword".to_string()));

    let formatted = format_multiple_results(&result);
    assert!(formatted.contains("**Attack**:"));
    assert!(formatted.contains("Reason: `With magic sword`"));

    // Test edge cases
    let label_edge_cases = vec![
        ("() 2d6", ""),                // Empty label
        ("(  ) 1d20", ""),             // Whitespace-only label
        ("( Attack ) 1d20", "Attack"), // Extra spaces should be trimmed
    ];

    for (expression, expected) in label_edge_cases {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(
            result[0].label,
            Some(expected.to_string()),
            "Edge case label for '{}'",
            expression
        );
    }
}

#[test]
fn test_special_flags_integration() {
    // Test special flags in complex integration scenarios

    // Test 'nr' (no results) flag with various systems
    let nr_tests = vec![
        ("nr 4d6 k3", "No results with keep modifier"),
        ("nr 3d10 t7", "No results with success counting"),
        ("nr sw8", "No results with Savage Worlds"),
        ("nr 4cod", "No results with Chronicles of Darkness"),
        ("nr 6 2d6", "No results with roll sets"),
    ];

    for (expression, description) in nr_tests {
        let result = parse_and_roll(expression).unwrap();

        for roll in &result {
            assert!(
                roll.no_results,
                "Should have no_results flag for '{}': {}",
                expression, description
            );
        }

        // Test formatted output - should show dice but not final results
        let formatted = format_multiple_results(&result);
        assert!(
            formatted.contains("`["),
            "Should show dice breakdown for nr flag: '{}'",
            expression
        );
        // Note: Actual formatting behavior for 'nr' flag needs to be verified based on implementation
    }

    // Each flag sets only itself, in any combination, and reaches every roll
    // of a roll set. (expression, private, simple, no results, unsorted)
    let flag_cases = vec![
        ("p s 2d6", true, true, false, false),
        ("p nr 2d6", true, false, true, false),
        ("s ul 2d6", false, true, false, true),
        ("nr ul 2d6", false, false, true, true),
        ("p s nr ul 2d6", true, true, true, true),
        ("p 3 2d6", true, false, false, false),
        ("s 2 1d20", false, true, false, false),
        ("ul 3 4d6", false, false, false, true),
        ("p s 5 4d6 k3 ! Character stats", true, true, false, false),
        ("p nr (Attack) 1d20 + 5", true, false, true, false),
    ];

    for (expression, private, simple, no_results, unsorted) in flag_cases {
        let results = parse_and_roll(expression)
            .unwrap_or_else(|e| panic!("'{expression}' should roll: {e}"));
        for roll in &results {
            assert_eq!(
                (roll.private, roll.simple, roll.no_results),
                (private, simple, no_results),
                "'{expression}' private/simple/no-results flags"
            );
        }
        // Unsorted only changes how the parsed dice are displayed
        for dice in parser::parse_dice_string(expression).unwrap() {
            assert_eq!(dice.unsorted, unsorted, "'{expression}' unsorted flag");
        }
    }
}

// ============================================================================
// ERROR REPLIES
// ============================================================================

#[test]
fn test_error_reply_fits_discord_limit() {
    let long_modifier = "z".repeat(990);
    let long_expr = format!("1d6 {long_modifier}");
    let unknown = format!("Unknown modifier: {long_modifier}");
    let huge_error = "é".repeat(1500); // multi-byte, so the cut must respect char boundaries

    // (expression, error, exact reply, or None when it is too long to send whole)
    let cases = vec![
        (
            "1d6 zz",
            "Unknown modifier: zz",
            Some("🎲 **Tester** used `1d6 zz` - ❌ **Error**: Unknown modifier: zz"),
        ),
        (
            "1d6",
            "Input too long: 1200 characters (max 1000)",
            Some("🎲 **Tester** ❌ **Error**: Input too long: 1200 characters (max 1000)"),
        ),
        (long_expr.as_str(), unknown.as_str(), None),
        ("1d6", huge_error.as_str(), None),
    ];

    for (expr, error, expected) in cases {
        let reply = error_reply("Tester", expr, error);
        assert!(
            reply.len() <= 2000,
            "reply for '{expr}' is {} bytes, over Discord's limit",
            reply.len()
        );
        match expected {
            Some(exact) => assert_eq!(reply, exact, "reply for '{expr}'"),
            None => assert!(
                reply.starts_with("🎲 **Tester** ❌ **Error**: "),
                "over-long reply for '{expr}' should drop the request: {reply}"
            ),
        }
    }
}

// ============================================================================
// PURGE
// ============================================================================

#[test]
fn test_purge_count_and_bot_permissions() {
    use dicemaiden_rs::commands::purge::{bot_can_purge, purge_count};
    use serenity::all::{CommandDataOptionValue, Permissions};

    // (count option, accepted count) — a cast once wrapped 300 to 44
    let counts = vec![
        (Some(CommandDataOptionValue::Integer(2)), Some(2)),
        (Some(CommandDataOptionValue::Integer(100)), Some(100)),
        (Some(CommandDataOptionValue::Integer(1)), None),
        (Some(CommandDataOptionValue::Integer(101)), None),
        (Some(CommandDataOptionValue::Integer(300)), None),
        (Some(CommandDataOptionValue::Integer(-50)), None),
        (Some(CommandDataOptionValue::Boolean(true)), None),
        (None, None),
    ];
    for (option, expected) in counts {
        assert_eq!(purge_count(option.as_ref()), expected, "count {option:?}");
    }

    // (bot's channel permissions, can purge)
    let manage_and_read = Permissions::MANAGE_MESSAGES | Permissions::READ_MESSAGE_HISTORY;
    let permissions = vec![
        (Some(manage_and_read), true),
        (Some(Permissions::ADMINISTRATOR), true),
        (Some(Permissions::MANAGE_MESSAGES), false),
        (Some(Permissions::READ_MESSAGE_HISTORY), false),
        (Some(Permissions::SEND_MESSAGES), false),
        (None, true), // not sent: try, and report what Discord refused
    ];
    for (perms, expected) in permissions {
        assert_eq!(bot_can_purge(perms), expected, "bot permissions {perms:?}");
    }
}

#[test]
fn test_format_memory_units() {
    use dicemaiden_rs::commands::roll::format_memory;
    // (megabytes, shown) — it switched at 1000 but divided by 1024
    let cases = vec![
        (68.69, "68.69 MB"),
        (1010.0, "1010.00 MB"),
        (1023.99, "1023.99 MB"),
        (1024.0, "1.00 GB"),
        (1167.36, "1.14 GB"),
    ];
    for (megabytes, shown) in cases {
        assert_eq!(format_memory(megabytes), shown, "{megabytes} MB");
    }
}

#[test]
fn test_roll_set_alias_behind_flags_label_and_comment() {
    // `dndstats` expands to the roll set `6 4d6 k3`; behind a flag, label or
    // comment it used to become one `6 + 4d6 k3` roll.
    for expression in [
        "dndstats",
        "dndstats ! new character",
        "p dndstats",
        "s dndstats",
        "(Stats) dndstats",
    ] {
        let rolls = parser::parse_dice_string(expression)
            .unwrap_or_else(|e| panic!("'{expression}' should parse: {e}"));
        assert_eq!(rolls.len(), 6, "'{expression}' should roll six stats");
        assert!(
            rolls.iter().all(|r| r.count == 4 && r.sides == 6),
            "'{expression}' should roll 4d6 each"
        );
    }
    // A set count around it would nest one roll set in another
    assert!(parser::parse_dice_string("3 dndstats").is_err());
}
