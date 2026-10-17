// tests/performance_tests.rs - Performance and Limit Tests
//
// This file contains tests for performance characteristics and boundary limits:
// - Explosion and reroll caps
// - Latency budgets for parsing and for the heaviest rolls
// - The regex-hoisting lint
//
// Hangs anywhere in the syntax are hunted by the on-demand fuzz test; these
// budgets guard the known worst cases on every run.

use dicemaiden_rs::parse_and_roll;
use std::time::Instant;

// ============================================================================
// EXPLOSION AND REROLL LIMITS
// ============================================================================

#[test]
fn test_explosion_limits() {
    // `ie1` on a d6 explodes on every face, so only the cap stops it: the
    // original die plus 100 explosions, and a note saying why it stopped
    for _ in 0..5 {
        let result = parse_and_roll("1d6 ie1").unwrap();
        assert_eq!(
            result[0].individual_rolls.len(),
            101,
            "Explosions should stop at the cap"
        );
        assert!(
            result[0]
                .notes
                .contains(&"Maximum explosions reached (100)".to_string()),
            "Hitting the cap should say so: {:?}",
            result[0].notes
        );
    }
}

#[test]
fn test_reroll_limits() {
    // Test that rerolls have reasonable limits

    // This should always reroll but be limited
    for _ in 0..5 {
        let result = parse_and_roll("3d6 ir6").unwrap();
        assert_eq!(
            result[0].individual_rolls.len(),
            3,
            "Should always have 3 dice"
        );

        // Should complete in reasonable time (covered by performance tests)
        // The limit is internal to prevent infinite loops
    }
}

// ============================================================================
// MEMORY USAGE AND OPTIMIZATION
// ============================================================================

// ============================================================================
// DISCORD MESSAGE LENGTH HANDLING
// ============================================================================

// ============================================================================
// STRESS TESTS
// ============================================================================

// ============================================================================
// REGEX COMPILATION REGRESSION GUARDS
// ============================================================================
//
// Background: every regex in `parser.rs` was once compiled on each call rather
// than once at startup. Compilation costs ~35us against ~1us of matching, and
// the parser evaluates ~20 patterns per input token, so a 966-character roll
// spent ~165ms of pure CPU rebuilding the same automata. On the gateway runtime
// that starved Serenity's shard runners until their heartbeats failed and the
// shards entered a reconnect loop.
//
// The two guards below protect the fix from opposite directions: the lint
// catches a reintroduced `Regex::new` in a function body statically, and the
// budget test catches any other cause of a large parsing slowdown.

/// Best-of-N wall time in microseconds. Best-of is used rather than mean
/// because it is far more robust to scheduler noise on shared CI runners.
fn best_parse_micros(expression: &str, runs: usize) -> u128 {
    let mut best = u128::MAX;
    for _ in 0..runs {
        let start = Instant::now();
        let _ = parse_and_roll(expression);
        best = best.min(start.elapsed().as_micros());
    }
    best
}

/// Rejects `Regex::new` anywhere outside a `static` initialiser in `src/`.
///
/// Regexes must be hoisted into `static ... : Lazy<Regex>` (or
/// `Lazy<Vec<Regex>>`) so they compile once per process instead of once per
/// call. This is a static check, so unlike a timing test it cannot flake.
///
/// The rule is expressed as an allowlist — a regex is legal *only* inside a
/// `static` item — rather than as a denylist of `fn` bodies. A denylist has to
/// enumerate every construct that can hold code (`fn`, `impl`, `trait`, a
/// `const` initialiser, a macro body) and silently misses whatever it forgets.
#[test]
fn test_regexes_are_hoisted_to_statics() {
    // Every source file that constructs regexes. Add new ones here.
    let sources = vec![
        ("dice/parser.rs", include_str!("../src/dice/parser.rs")),
        ("dice/aliases.rs", include_str!("../src/dice/aliases.rs")),
        ("dice/mod.rs", include_str!("../src/dice/mod.rs")),
        ("commands/roll.rs", include_str!("../src/commands/roll.rs")),
    ];

    let mut offenders: Vec<String> = Vec::new();

    for (path, source) in sources {
        // rustfmt guarantees top-level items begin at column 0, so a `static`
        // item runs from its own line until the next unindented line.
        let mut in_static_item = false;

        for (index, line) in source.lines().enumerate() {
            let starts_new_item = !line.is_empty() && !line.starts_with(char::is_whitespace);
            if starts_new_item {
                in_static_item = line.starts_with("static ") || line.starts_with("pub static ");
            }

            if !in_static_item && line.contains("Regex::new") {
                offenders.push(format!("  {}:{}: {}", path, index + 1, line.trim()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "regexes must be compiled once, in a `static NAME: Lazy<Regex>`, not per call.\n\
         Offending lines:\n{}",
        offenders.join("\n")
    );
}

/// Backstop for the lint: asserts parsing stays far below the latency that
/// caused the shard-starvation incident.
///
/// Budgets are deliberately loose. Measured worst case across a 797-expression
/// corpus is ~0.26ms, and the pre-fix worst case was ~18.8ms, so 5ms both
/// leaves ~19x headroom against CI noise and still fails loudly on a real
/// regression.
#[test]
fn test_parsing_stays_within_latency_budget() {
    const BUDGET_MICROS: u128 = 5_000;

    let cases = vec![
        ("1d20", "minimal roll"),
        ("4d6 k3", "common stat roll"),
        ("10d10 e10 k5 +3", "several modifiers"),
        ("10d6 ie6 k5 r1 t4 +5", "modifier-dense"),
        ("1d6 + 1d6 t4f1ie6 + 1d6 e6k1", "worst case in corpus"),
        ("4d6 k3 ; 4d6 k3 ; 4d6 k3", "semicolon-separated"),
        ("6 4d6 k3", "roll set"),
        ("4d6 k3 ! character generation", "with comment"),
    ];

    for (expression, description) in cases {
        let elapsed = best_parse_micros(expression, 20);
        assert!(
            elapsed <= BUDGET_MICROS,
            "parsing '{expression}' ({description}) took {elapsed}us, budget is {BUDGET_MICROS}us. \
             This usually means a regex is being compiled per call rather than in a Lazy static."
        );
    }
}

/// The heaviest rolls the syntax allows, which parse quickly but roll, explode
/// or format a lot. They answer the same Discord interaction, so they get a
/// budget too, looser than parsing's because they really do the work.
///
/// Measured worst case in a debug build is ~0.6ms (`20 50d6`), so 50ms leaves
/// ~80x headroom for CI noise while still catching a roll that loops.
#[test]
fn test_worst_case_rolls_stay_within_latency_budget() {
    const BUDGET_MICROS: u128 = 50_000;

    let cases = vec![
        ("500d1000", "most dice allowed"),
        ("20 50d6", "large roll set"),
        ("100d6 e6 ie k50 r1 t4 +10", "every modifier family"),
        ("1d6 ie1", "explodes until the cap"),
        ("50d6 k25 d10", "keep and drop on a big pool"),
        ("20d10 t1 f10 b1", "every die counted three ways"),
        ("5d10 f1 t6 c ie10 + 2", "cancel with explosions"),
        ("20 4wod8c", "roll set of cancel rolls"),
        ("alien10s10", "maximum Alien stress"),
        ("20 alien3s2", "roll set of Alien stress rolls"),
        ("4d6;3d8;2d10;1d20", "most semicolon rolls"),
    ];

    for (expression, description) in cases {
        let elapsed = best_parse_micros(expression, 5);
        assert!(
            elapsed <= BUDGET_MICROS,
            "rolling '{expression}' ({description}) took {elapsed}us, budget is {BUDGET_MICROS}us"
        );
    }
}
