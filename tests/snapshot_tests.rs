// tests/snapshot_tests.rs - Golden-snapshot regression tests
//
// Dice syntax is a public API and a silently changed roll result is the worst
// possible bug this project can ship. These tests pin the observable behavior of
// every expression in `tests/corpus/expressions.dice` so that a refactor which
// changes a result fails here instead of on 500,000 Discord servers.
//
// Three layers, cheapest and strictest first:
//
// 1. `parse_snapshot`   — parsing is fully deterministic, so the parsed form of
//                         every expression (and every parse error) is pinned
//                         exactly.
// 2. `roll_snapshot`    — expressions whose dice cannot vary (`d1` pools and
//                         friends) produce a fixed result, so the full roll
//                         outcome *and* the formatted Discord output are pinned
//                         exactly.
// 3. `outcome_snapshot` — for expressions that do roll randomly, the parts that
//                         genuinely cannot vary with the dice are pinned: how
//                         many results come back, whether the expression
//                         succeeds or fails, and the exact error text when it
//                         always fails.
//
// Note what layer 3 deliberately does NOT assert: which optional fields a result
// populates is *not* an invariant of the expression. `4d6 i1` only grows a
// "subtract" dice group when an implosion actually fires, and Mothership sets
// `successes` or `failures` depending on how the roll went. Pinning those would
// be pinning the dice. Exact coverage of random systems needs a seedable RNG.
//
// To update after an intentional behavior change:
//
//     UPDATE_SNAPSHOTS=1 cargo test --test snapshot_tests
//
// Then read the diff. Every changed line is a change your users will see.

use dicemaiden_rs::{RollResult, dice::parser, format_multiple_results, parse_and_roll};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Trials used to confirm a "deterministic" expression really is fixed, and to
/// confirm a random expression's result shape never varies.
const STABILITY_TRIALS: usize = 25;

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn corpus() -> Vec<String> {
    let path = repo_path("tests/corpus/expressions.dice");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect()
}

/// Compare `actual` against the checked-in snapshot, or rewrite it when
/// `UPDATE_SNAPSHOTS=1` is set.
fn assert_snapshot(name: &str, actual: &str) {
    let path = repo_path(&format!("tests/snapshots/{name}"));

    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&path, actual)
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read snapshot {}: {e}\n\
             Create it with: UPDATE_SNAPSHOTS=1 cargo test --test snapshot_tests",
            path.display()
        )
    });

    if expected == actual {
        return;
    }

    let mut report = String::new();
    let mut shown = 0;
    for (line_no, (want, got)) in expected.lines().zip(actual.lines()).enumerate() {
        if want != got && shown < 10 {
            let _ = write!(
                report,
                "\n  line {}:\n    expected: {want}\n    actual:   {got}",
                line_no + 1
            );
            shown += 1;
        }
    }
    if expected.lines().count() != actual.lines().count() {
        let _ = write!(
            report,
            "\n  line count changed: {} -> {}",
            expected.lines().count(),
            actual.lines().count()
        );
    }

    panic!(
        "Snapshot `{name}` no longer matches — behavior visible to users changed.\n\
         If the change is intentional, regenerate and review the diff:\n\
         \x20   UPDATE_SNAPSHOTS=1 cargo test --test snapshot_tests\n{report}"
    );
}

/// The parsed form of an expression, projected to the parts that decide how it
/// rolls. Deliberately not `{:?}` of the whole struct, so that adding an
/// internal field does not churn every line of the snapshot.
fn parse_projection(expression: &str) -> String {
    match parser::parse_dice_string(expression) {
        Err(e) => format!("{expression}\tERR\t{e}"),
        Ok(rolls) => {
            let parts: Vec<String> = rolls
                .iter()
                .map(|dice| {
                    format!(
                        "{}d{} mods={:?} label={:?} comment={:?} flags={}{}{}{}",
                        dice.count,
                        dice.sides,
                        dice.modifiers,
                        dice.label,
                        dice.comment,
                        if dice.private { "p" } else { "-" },
                        if dice.simple { "s" } else { "-" },
                        if dice.no_results { "n" } else { "-" },
                        if dice.unsorted { "u" } else { "-" },
                    )
                })
                .collect();
            format!("{expression}\tOK\t[{}]", parts.join(" | "))
        }
    }
}

/// Everything a user can observe about one rolled result.
fn roll_projection(result: &RollResult) -> String {
    format!(
        "total={} successes={:?} failures={:?} botches={:?} kept={:?} dropped={:?} individual={:?} \
         implosions={:?} groups={:?} notes={:?} label={:?} comment={:?} godbound={:?} fudge={:?} \
         plot={:?} fitd={:?}/{:?}/{:?} wng={:?}/{:?}/{:?} alien={:?}/{:?}/{:?}",
        result.total,
        result.successes,
        result.failures,
        result.botches,
        result.kept_rolls,
        result.dropped_rolls,
        result.individual_rolls,
        result.implosion_rolls,
        result
            .dice_groups
            .iter()
            .map(|g| (
                g.modifier_type.as_str(),
                g.rolls.clone(),
                g.dropped_rolls.clone()
            ))
            .collect::<Vec<_>>(),
        result.notes,
        result.label,
        result.comment,
        result.godbound_damage,
        result.fudge_symbols,
        result.plot_symbols,
        result.fitd_outcome,
        result.fitd_result,
        result.fitd_highest_die,
        result.wng_icons,
        result.wng_exalted_icons,
        result.wng_wrath_dice,
        result.alien_stress_level,
        result.alien_panic_roll,
        result.alien_stress_ones,
    )
}

/// Roll an expression `STABILITY_TRIALS` times; return `Some(output)` only if
/// every trial produced byte-identical output.
fn stable_roll_output(expression: &str) -> Option<String> {
    let render = || {
        parse_and_roll(expression).ok().map(|results| {
            let rolls: Vec<String> = results.iter().map(roll_projection).collect();
            format!(
                "{}\t{}\tFMT\t{}",
                expression,
                rolls.join(" || "),
                format_multiple_results(&results).replace('\n', "\\n")
            )
        })
    };

    let first = render()?;
    for _ in 1..STABILITY_TRIALS {
        if render()? != first {
            return None;
        }
    }
    Some(first)
}

#[test]
fn parse_snapshot_is_unchanged() {
    let lines: Vec<String> = corpus().iter().map(|e| parse_projection(e)).collect();
    assert_snapshot("parse.snap", &format!("{}\n", lines.join("\n")));
}

#[test]
fn deterministic_roll_snapshot_is_unchanged() {
    // Only expressions whose dice cannot vary land here. An expression that
    // stops being deterministic drops out of the snapshot, which changes the
    // line count and fails the test — that is the intended alarm.
    let lines: Vec<String> = corpus()
        .iter()
        .filter_map(|e| stable_roll_output(e))
        .collect();
    assert!(
        lines.len() > 200,
        "expected the deterministic corpus to stay substantial, got {} entries",
        lines.len()
    );
    assert_snapshot(
        "deterministic_rolls.snap",
        &format!("{}\n", lines.join("\n")),
    );
}

#[test]
fn outcome_snapshot_is_unchanged() {
    let mut lines = Vec::new();

    for expression in corpus() {
        let mut ok_trials = 0usize;
        let mut counts = std::collections::BTreeSet::new();
        let mut errors = std::collections::BTreeSet::new();

        for _ in 0..STABILITY_TRIALS {
            match parse_and_roll(&expression) {
                Ok(results) => {
                    ok_trials += 1;
                    counts.insert(results.len());
                }
                Err(e) => {
                    errors.insert(e.to_string());
                }
            }
        }

        // How many results an expression yields is fixed by its syntax (a roll
        // set of 3 is always 3), never by the dice.
        assert!(
            counts.len() <= 1,
            "'{expression}' returned differing result counts across trials: {counts:?}"
        );

        let outcome = if ok_trials == STABILITY_TRIALS {
            format!("always-ok n={:?}", counts.iter().next())
        } else if ok_trials == 0 {
            format!("always-err {errors:?}")
        } else {
            // Legitimate for expressions that can divide by a rolled zero.
            format!("varies ok={ok_trials}/{STABILITY_TRIALS} errs={errors:?}")
        };

        lines.push(format!("{expression}\t{outcome}"));
    }

    assert_snapshot("outcomes.snap", &format!("{}\n", lines.join("\n")));
}
